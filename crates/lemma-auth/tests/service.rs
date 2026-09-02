#![allow(clippy::unwrap_used, missing_docs)]

use buffa::Message;
use connectrpc::{
    CodecFormat, Encodable, ErrorCode, HasMessageView, JsonSerialize, RequestContext,
    ServiceRequest,
};
use http::HeaderMap;
use lemma_auth::AuthService;
use lemma_proto::lemma::v1::AuthService as AuthServiceRpc;
use lemma_proto::lemma::v1::{
    LoginRequest, LoginResponse, LogoutRequest, LogoutResponse, MeRequest, MeResponse,
    RefreshRequest, RefreshResponse, Role, SignUpRequest, SignUpResponse,
};
use sqlx::PgPool;

const SECRET: &str = "test-secret";
const PASSWORD: &str = "password123";

fn owned_body<M>(body: &impl Encodable<M>) -> M
where
    M: Message + JsonSerialize,
{
    let bytes = body.encode(CodecFormat::Proto).unwrap();
    M::decode(&mut &bytes[..]).unwrap()
}

async fn signup(
    svc: &AuthService,
    username: &str,
    email: &str,
    password: &str,
) -> Result<SignUpResponse, connectrpc::ConnectError> {
    let msg = SignUpRequest {
        username: username.into(),
        email: email.into(),
        password: password.into(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = SignUpRequest::decode_view(&bytes).unwrap();
    match svc
        .sign_up(
            RequestContext::new(HeaderMap::new()),
            ServiceRequest::from_parts(&view, &bytes),
        )
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

async fn login(
    svc: &AuthService,
    username: &str,
    email: &str,
    password: &str,
) -> Result<LoginResponse, connectrpc::ConnectError> {
    let msg = LoginRequest {
        username: username.into(),
        email: email.into(),
        password: password.into(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = LoginRequest::decode_view(&bytes).unwrap();
    match svc
        .login(
            RequestContext::new(HeaderMap::new()),
            ServiceRequest::from_parts(&view, &bytes),
        )
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

async fn refresh(
    svc: &AuthService,
    token: &str,
) -> Result<RefreshResponse, connectrpc::ConnectError> {
    let msg = RefreshRequest {
        refresh_token: token.into(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = RefreshRequest::decode_view(&bytes).unwrap();
    match svc
        .refresh(
            RequestContext::new(HeaderMap::new()),
            ServiceRequest::from_parts(&view, &bytes),
        )
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

async fn logout(
    svc: &AuthService,
    token: &str,
) -> Result<LogoutResponse, connectrpc::ConnectError> {
    let msg = LogoutRequest {
        refresh_token: token.into(),
        ..Default::default()
    };
    let bytes = msg.encode_to_bytes();
    let view = LogoutRequest::decode_view(&bytes).unwrap();
    match svc
        .logout(
            RequestContext::new(HeaderMap::new()),
            ServiceRequest::from_parts(&view, &bytes),
        )
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

fn bearer_ctx(token: &str) -> RequestContext {
    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {token}").parse().unwrap(),
    );
    RequestContext::new(headers)
}

async fn me(
    svc: &AuthService,
    token: Option<&str>,
) -> Result<MeResponse, connectrpc::ConnectError> {
    let ctx = match token {
        Some(t) => bearer_ctx(t),
        None => RequestContext::new(HeaderMap::new()),
    };
    let msg = MeRequest::default();
    let bytes = msg.encode_to_bytes();
    let view = MeRequest::decode_view(&bytes).unwrap();
    match svc.me(ctx, ServiceRequest::from_parts(&view, &bytes)).await {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn signup_first_owner_second_normal(pool: PgPool) {
    let svc = AuthService::new(pool, SECRET);
    let r1 = signup(&svc, "alice", "alice@example.com", PASSWORD)
        .await
        .unwrap();
    assert_eq!(r1.user.role, Role::Owner);
    let r2 = signup(&svc, "bob", "bob@example.com", PASSWORD)
        .await
        .unwrap();
    assert_eq!(r2.user.role, Role::Normal);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn signup_duplicate_username_rejected(pool: PgPool) {
    let svc = AuthService::new(pool, SECRET);
    signup(&svc, "alice", "alice@example.com", PASSWORD)
        .await
        .unwrap();
    let err = signup(&svc, "alice", "other@example.com", PASSWORD)
        .await
        .err()
        .unwrap();
    assert_eq!(err.code, ErrorCode::InvalidArgument);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn login_ok_and_wrong_password(pool: PgPool) {
    let svc = AuthService::new(pool, SECRET);
    signup(&svc, "alice", "alice@example.com", PASSWORD)
        .await
        .unwrap();
    let r = login(&svc, "alice", "", PASSWORD).await.unwrap();
    assert_eq!(r.user.username, "alice");
    let err = login(&svc, "alice", "", "wrongpass1").await.err().unwrap();
    assert_eq!(err.code, ErrorCode::Unauthenticated);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn login_requires_exactly_one_identifier(pool: PgPool) {
    let svc = AuthService::new(pool, SECRET);
    let err = login(&svc, "alice", "alice@example.com", PASSWORD)
        .await
        .err()
        .unwrap();
    assert_eq!(err.code, ErrorCode::InvalidArgument);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn refresh_replay_revokes_whole_chain(pool: PgPool) {
    let svc = AuthService::new(pool, SECRET);
    let tokens = signup(&svc, "alice", "alice@example.com", PASSWORD)
        .await
        .unwrap()
        .tokens
        .refresh_token
        .clone();

    let rotated = refresh(&svc, &tokens)
        .await
        .unwrap()
        .tokens
        .refresh_token
        .clone();
    let err = refresh(&svc, &tokens).await.err().unwrap();
    assert_eq!(err.code, ErrorCode::Unauthenticated);
    let err = refresh(&svc, &rotated).await.err().unwrap();
    assert_eq!(err.code, ErrorCode::Unauthenticated);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn logout_is_idempotent(pool: PgPool) {
    let svc = AuthService::new(pool, SECRET);
    logout(&svc, "nonexistent").await.unwrap();
    let tokens = signup(&svc, "alice", "alice@example.com", PASSWORD)
        .await
        .unwrap()
        .tokens
        .refresh_token
        .clone();
    logout(&svc, &tokens).await.unwrap();
    logout(&svc, &tokens).await.unwrap();
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn me_requires_valid_bearer(pool: PgPool) {
    let svc = AuthService::new(pool, SECRET);
    let access = signup(&svc, "alice", "alice@example.com", PASSWORD)
        .await
        .unwrap()
        .tokens
        .access_token
        .clone();

    let r = me(&svc, Some(&access)).await.unwrap();
    assert_eq!(r.user.username, "alice");

    let err = me(&svc, None).await.err().unwrap();
    assert_eq!(err.code, ErrorCode::Unauthenticated);
    let err = me(&svc, Some("garbage")).await.err().unwrap();
    assert_eq!(err.code, ErrorCode::Unauthenticated);
}
