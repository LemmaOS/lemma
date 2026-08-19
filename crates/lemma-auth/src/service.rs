use buffa_types::google::protobuf::Timestamp;
use chrono::{Duration, Utc};
use connectrpc::{ConnectError, RequestContext, Response, ServiceRequest, ServiceResult};
use lemma_db::entity::User as DbUser;
use crate::{tokens, users};
use lemma_proto::lemma::v1::{
    AuthTokens, LoginResponse, LogoutResponse, MeResponse, RefreshResponse, Role, SignUpResponse,
    User,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::jwt::ACCESS_TOKEN_TTL_SECS;
use crate::{
    generate_refresh_token, hash_password, hash_token, sign_access_token, verify_password,
};

// refresh token 寿命 30 天（滑动）
const REFRESH_TTL_DAYS: i64 = 30;

pub struct AuthService {
    pool: PgPool,
    secret: String,
}

impl AuthService {
    pub fn new(pool: PgPool, secret: impl Into<String>) -> Self {
        Self {
            pool,
            secret: secret.into(),
        }
    }

    // 签发令牌对：access 走 JWT，refresh 落库存哈希；返回 refresh 行 id 供轮换关联
    async fn issue_tokens<'e, E>(
        &self,
        executor: E,
        user_id: Uuid,
    ) -> Result<(AuthTokens, Uuid), ConnectError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let access = sign_access_token(&self.secret, user_id)
            .map_err(|e| ConnectError::internal(format!("jwt sign: {e}")))?;
        let refresh = generate_refresh_token();
        let refresh_id = Uuid::new_v4();
        tokens::insert(
            executor,
            refresh_id,
            user_id,
            &hash_token(&refresh),
            None,
            Utc::now() + Duration::days(REFRESH_TTL_DAYS),
        )
        .await
        .map_err(map_db)?;
        let exp = Utc::now() + Duration::seconds(ACCESS_TOKEN_TTL_SECS);
        let tokens = AuthTokens {
            access_token: access,
            access_token_expires_at: Timestamp::from(exp).into(),
            refresh_token: refresh,
            ..Default::default()
        };
        Ok((tokens, refresh_id))
    }
}

fn user_to_proto(u: &DbUser) -> User {
    User {
        id: u.id.to_string(),
        username: u.username.clone(),
        email: u.email.clone(),
        role: match u.role.as_str() {
            "owner" => Role::Owner,
            _ => Role::Normal,
        }
        .into(),
        created_at: Timestamp::from(u.created_at).into(),
        ..Default::default()
    }
}

// db 错误到 Connect 错误的统一映射
fn map_db(e: sqlx::Error) -> ConnectError {
    match &e {
        sqlx::Error::Database(d) if d.is_unique_violation() => {
            ConnectError::invalid_argument("username or email already taken")
        }
        _ => ConnectError::internal(format!("db: {e}")),
    }
}

// 是否撞 owner 唯一索引（并发注册竞态）
fn is_owner_conflict(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .and_then(|d| d.constraint())
        .is_some_and(|c| c == "users_owner_unique")
}

#[allow(refining_impl_trait)]
impl lemma_proto::lemma::v1::AuthService for AuthService {
    async fn sign_up(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::SignUpRequest>,
    ) -> ServiceResult<SignUpResponse> {
        let username = request.username.trim();
        let email = request.email.trim();
        let password = request.password;
        if username.is_empty() || email.is_empty() || password.len() < 8 {
            return Err(ConnectError::invalid_argument(
                "username and email required, password at least 8 chars",
            ));
        }
        let hash = hash_password(password)
            .map_err(|e| ConnectError::internal(format!("hash password: {e}")))?;
        let user = match users::insert(&self.pool, username, email, &hash).await {
            Ok(u) => u,
            // 并发首批注册撞 owner 索引：按 normal 重试一次
            Err(e) if is_owner_conflict(&e) => users::insert(&self.pool, username, email, &hash)
                .await
                .map_err(map_db)?,
            Err(e) => return Err(map_db(e)),
        };
        let (tokens, _) = self.issue_tokens(&self.pool, user.id).await?;
        Response::ok(SignUpResponse {
            user: user_to_proto(&user).into(),
            tokens: tokens.into(),
            ..Default::default()
        })
    }

    async fn login(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::LoginRequest>,
    ) -> ServiceResult<LoginResponse> {
        let username = request.username.trim();
        let email = request.email.trim();
        let login = match (username.is_empty(), email.is_empty()) {
            (false, true) => username,
            (true, false) => email,
            _ => {
                return Err(ConnectError::invalid_argument(
                    "provide exactly one of username or email",
                ));
            }
        };
        let user = users::find_by_login(&self.pool, login)
            .await
            .map_err(map_db)?
            .ok_or_else(|| ConnectError::unauthenticated("invalid credentials"))?;
        if !verify_password(request.password, &user.password_hash) {
            return Err(ConnectError::unauthenticated("invalid credentials"));
        }
        let (tokens, _) = self.issue_tokens(&self.pool, user.id).await?;
        Response::ok(LoginResponse {
            user: user_to_proto(&user).into(),
            tokens: tokens.into(),
            ..Default::default()
        })
    }

    async fn refresh(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::RefreshRequest>,
    ) -> ServiceResult<RefreshResponse> {
        let hash = hash_token(request.refresh_token);
        let row = tokens::find_by_hash(&self.pool, &hash)
            .await
            .map_err(map_db)?
            .ok_or_else(|| ConnectError::unauthenticated("invalid refresh token"))?;
        // 已吊销/已轮换的 token 再出现 = 泄露，整链吊销
        if row.revoked_at.is_some() || row.replaced_by.is_some() {
            let _ = tokens::revoke_chain(&self.pool, row.id).await;
            return Err(ConnectError::unauthenticated(
                "refresh token replay detected",
            ));
        }
        if row.expires_at <= Utc::now() {
            return Err(ConnectError::unauthenticated("refresh token expired"));
        }
        // 事务内轮换：插新 token + 标记旧 token
        let mut tx = self.pool.begin().await.map_err(map_db)?;
        let (new_tokens, new_id) = self.issue_tokens(&mut *tx, row.user_id).await?;
        tokens::mark_replaced(&mut *tx, row.id, new_id)
            .await
            .map_err(map_db)?;
        tx.commit().await.map_err(map_db)?;
        Response::ok(RefreshResponse {
            tokens: new_tokens.into(),
            ..Default::default()
        })
    }

    async fn logout(
        &self,
        _ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::LogoutRequest>,
    ) -> ServiceResult<LogoutResponse> {
        let hash = hash_token(request.refresh_token);
        if let Some(row) = tokens::find_by_hash(&self.pool, &hash)
            .await
            .map_err(map_db)?
        {
            tokens::revoke(&self.pool, row.id).await.map_err(map_db)?;
        }
        Response::ok(LogoutResponse::default())
    }

    async fn me(
        &self,
        ctx: RequestContext,
        _request: ServiceRequest<'_, lemma_proto::lemma::v1::MeRequest>,
    ) -> ServiceResult<MeResponse> {
        let user_id = crate::require_user(&self.secret, &ctx)?;
        let user = users::find_by_id(&self.pool, user_id)
            .await
            .map_err(map_db)?
            .ok_or_else(|| ConnectError::not_found("user not found"))?;
        Response::ok(MeResponse {
            user: user_to_proto(&user).into(),
            ..Default::default()
        })
    }
}
