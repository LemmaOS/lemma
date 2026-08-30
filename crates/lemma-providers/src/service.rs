use crate::providers::{self, NewProvider, ProviderPatch};
use buffa::EnumValue;
use buffa_types::google::protobuf::Timestamp;
use connectrpc::{ConnectError, RequestContext, Response, ServiceRequest, ServiceResult};
use lemma_db::entity::Provider as DbProvider;
use lemma_proto::app_error;
use lemma_proto::lemma::v1::{
    CreateProviderResponse, DeleteProviderResponse, ErrorReason, FetchModelsResponse,
    ListProvidersResponse, Provider, ProviderKind, UpdateProviderResponse,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::fetch_models;
use lemma_crypto::{derive_key, mask, open, seal};

pub struct ProviderService {
    pool: PgPool,
    jwt_secret: String,
    secret_key: String,
}

impl ProviderService {
    pub fn new(pool: PgPool, jwt_secret: impl Into<String>, secret_key: impl Into<String>) -> Self {
        Self {
            pool,
            jwt_secret: jwt_secret.into(),
            secret_key: secret_key.into(),
        }
    }
}

/// DB 存的 kind 字符串 → proto 枚举（未知值宽松兜底为 openai）
pub fn kind_to_proto(kind: &str) -> ProviderKind {
    match kind {
        "anthropic" => ProviderKind::Anthropic,
        "gemini" => ProviderKind::Gemini,
        _ => ProviderKind::Openai,
    }
}

fn kind_from_proto(kind: &EnumValue<ProviderKind>) -> Result<&'static str, ConnectError> {
    match kind.as_known() {
        Some(ProviderKind::Anthropic) => Ok("anthropic"),
        Some(ProviderKind::Gemini) => Ok("gemini"),
        Some(ProviderKind::Openai) => Ok("openai"),
        _ => Err(app_error(ErrorReason::ProviderKindInvalid)),
    }
}

// 脱敏需要真实 key 的首尾，所以先解密再遮
fn to_proto(p: &DbProvider, secret_key: &str) -> Provider {
    let key = derive_key(secret_key);
    let api_key = open(&key, &p.api_key)
        .map(|k| mask(&k))
        .unwrap_or_else(|_| "****".to_string());
    Provider {
        id: p.id.to_string(),
        kind: kind_to_proto(&p.kind).into(),
        name: p.name.clone(),
        base_url: p.base_url.clone(),
        api_key,
        models: p.models.0.clone(),
        enabled: p.enabled,
        api_path: p.api_path.clone(),
        models_path: p.models_path.clone(),
        created_at: Timestamp::from(p.created_at).into(),
        updated_at: Timestamp::from(p.updated_at).into(),
        ..Default::default()
    }
}

fn parse_id(id: &str) -> Result<Uuid, ConnectError> {
    Uuid::parse_str(id).map_err(|_| app_error(ErrorReason::IdInvalid))
}

fn map_db(e: sqlx::Error) -> ConnectError {
    ConnectError::internal(format!("db: {e}"))
}

#[allow(refining_impl_trait)]
impl lemma_proto::lemma::v1::ProviderService for ProviderService {
    async fn list_providers(
        &self,
        ctx: RequestContext,
        _request: ServiceRequest<'_, lemma_proto::lemma::v1::ListProvidersRequest>,
    ) -> ServiceResult<ListProvidersResponse> {
        let user_id = lemma_auth::require_user(&self.jwt_secret, &ctx)?;
        let list = providers::list_by_user(&self.pool, user_id)
            .await
            .map_err(map_db)?;
        Response::ok(ListProvidersResponse {
            providers: list.iter().map(|p| to_proto(p, &self.secret_key)).collect(),
            ..Default::default()
        })
    }

    async fn create_provider(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::CreateProviderRequest>,
    ) -> ServiceResult<CreateProviderResponse> {
        let user_id = lemma_auth::require_user(&self.jwt_secret, &ctx)?;
        let kind = kind_from_proto(&request.kind)?;
        let name = request.name.trim();
        let base_url = request.base_url.trim().trim_end_matches('/');
        let api_key = request.api_key;
        if name.is_empty() || base_url.is_empty() || api_key.is_empty() {
            return Err(app_error(ErrorReason::ProviderFieldsRequired));
        }
        let sealed = {
            let key = derive_key(&self.secret_key);
            seal(&key, api_key)
        }
        .map_err(|e| ConnectError::internal(format!("seal key: {e}")))?;
        let provider = providers::insert(
            &self.pool,
            &NewProvider {
                id: Uuid::new_v4(),
                user_id,
                kind,
                name,
                base_url,
                api_key: &sealed,
                api_path: request.api_path.trim(),
                models_path: request.models_path.trim(),
                models: &request
                    .models
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            },
        )
        .await
        .map_err(map_db)?;
        Response::ok(CreateProviderResponse {
            provider: to_proto(&provider, &self.secret_key).into(),
            ..Default::default()
        })
    }

    async fn update_provider(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::UpdateProviderRequest>,
    ) -> ServiceResult<UpdateProviderResponse> {
        let user_id = lemma_auth::require_user(&self.jwt_secret, &ctx)?;
        let id = parse_id(request.id)?;
        // optional 标量：None = 不变更；api_key 额外把空串视为不变更
        let api_key = match request.api_key {
            Some(k) if !k.is_empty() => {
                let key = derive_key(&self.secret_key);
                Some(seal(&key, k).map_err(|e| ConnectError::internal(format!("seal key: {e}")))?)
            }
            _ => None,
        };
        let patch = ProviderPatch {
            name: request.name.map(|s| s.trim().to_string()),
            base_url: request
                .base_url
                .map(|s| s.trim().trim_end_matches('/').to_string()),
            api_key,
            api_path: request.api_path.map(|s| s.trim().to_string()),
            models_path: request.models_path.map(|s| s.trim().to_string()),
            enabled: request.enabled,
            models: request
                .models
                .as_option()
                .map(|m| m.models.iter().map(|s| s.to_string()).collect()),
        };
        let provider = providers::update(&self.pool, id, user_id, patch)
            .await
            .map_err(map_db)?
            .ok_or_else(|| app_error(ErrorReason::ProviderNotFound))?;
        Response::ok(UpdateProviderResponse {
            provider: to_proto(&provider, &self.secret_key).into(),
            ..Default::default()
        })
    }

    async fn delete_provider(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::DeleteProviderRequest>,
    ) -> ServiceResult<DeleteProviderResponse> {
        let user_id = lemma_auth::require_user(&self.jwt_secret, &ctx)?;
        let id = parse_id(request.id)?;
        let deleted = providers::delete(&self.pool, id, user_id)
            .await
            .map_err(map_db)?;
        if !deleted {
            return Err(app_error(ErrorReason::ProviderNotFound));
        }
        Response::ok(DeleteProviderResponse::default())
    }

    async fn fetch_models(
        &self,
        ctx: RequestContext,
        request: ServiceRequest<'_, lemma_proto::lemma::v1::FetchModelsRequest>,
    ) -> ServiceResult<FetchModelsResponse> {
        let user_id = lemma_auth::require_user(&self.jwt_secret, &ctx)?;
        let (kind, base_url, api_key, models_path) = if !request.id.is_empty() {
            // 已存配置模式：解密后调用
            let id = parse_id(request.id)?;
            let p = providers::find_by_id_and_user(&self.pool, id, user_id)
                .await
                .map_err(map_db)?
                .ok_or_else(|| app_error(ErrorReason::ProviderNotFound))?;
            let key = derive_key(&self.secret_key);
            let plain = open(&key, &p.api_key)
                .map_err(|e| ConnectError::internal(format!("open key: {e}")))?;
            (
                kind_to_proto(&p.kind),
                p.base_url.clone(),
                plain,
                p.models_path.clone(),
            )
        } else {
            // 临时凭证模式
            (
                kind_to_proto(kind_from_proto(&request.kind)?),
                request.base_url.trim().trim_end_matches('/').to_string(),
                request.api_key.to_string(),
                request.models_path.trim().to_string(),
            )
        };
        let models = fetch_models(kind, &base_url, &api_key, &models_path)
            .await
            .map_err(|e| ConnectError::internal(format!("fetch models: {e}")))?;
        Response::ok(FetchModelsResponse {
            models,
            ..Default::default()
        })
    }
}
