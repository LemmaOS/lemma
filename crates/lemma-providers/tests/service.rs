#![allow(clippy::unwrap_used)]

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use buffa::Message;
use connectrpc::{
    CodecFormat, Encodable, ErrorCode, HasMessageView, JsonSerialize, RequestContext,
    ServiceRequest,
};
use http::HeaderMap;
use lemma_auth::sign_access_token;
use lemma_proto::lemma::v1::ProviderService as ProviderServiceRpc;
use lemma_proto::lemma::v1::{
    CreateProviderRequest, CreateProviderResponse, DeleteProviderRequest, DeleteProviderResponse,
    ListProvidersRequest, ListProvidersResponse, ProviderKind, UpdateProviderRequest,
    UpdateProviderResponse,
};
use lemma_proto::lemma::v1::{FetchModelsRequest, FetchModelsResponse};
use lemma_providers::ProviderService;
use lemma_providers::providers as store;
use sqlx::PgPool;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

const SECRET: &str = "test-secret";
const KEY_SECRET: &str = "key-secret";

// 经 wire 编解码还原具体消息：rustc 走 M: Encodable<M> 自反实现，rust-analyzer 走
// 不透明类型的 Encodable<M> 参数化，两侧推导一致，绕开 RA 对 RPITIT 精化的误报
fn owned_body<M>(body: &impl Encodable<M>) -> M
where
    M: Message + JsonSerialize,
{
    let bytes = body.encode(CodecFormat::Proto).unwrap();
    M::decode(&mut &bytes[..]).unwrap()
}

// 直签 access token 注入 Bearer 头，省去走注册流程
fn auth_ctx(user_id: Uuid) -> RequestContext {
    let mut headers = HeaderMap::new();
    let token = sign_access_token(SECRET, user_id).unwrap();
    headers.insert(
        http::header::AUTHORIZATION,
        format!("Bearer {token}").parse().unwrap(),
    );
    RequestContext::new(headers)
}

async fn new_user(pool: &PgPool, name: &str) -> Uuid {
    lemma_auth::users::insert(pool, name, &format!("{name}@example.com"), "hash")
        .await
        .unwrap()
        .id
}

async fn list(
    svc: &ProviderService,
    ctx: RequestContext,
) -> Result<ListProvidersResponse, connectrpc::ConnectError> {
    let msg = ListProvidersRequest::default();
    let bytes = msg.encode_to_bytes();
    let view = ListProvidersRequest::decode_view(&bytes).unwrap();
    match svc
        .list_providers(ctx, ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

async fn create(
    svc: &ProviderService,
    ctx: RequestContext,
    msg: CreateProviderRequest,
) -> Result<CreateProviderResponse, connectrpc::ConnectError> {
    let bytes = msg.encode_to_bytes();
    let view = CreateProviderRequest::decode_view(&bytes).unwrap();
    match svc
        .create_provider(ctx, ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

async fn update(
    svc: &ProviderService,
    ctx: RequestContext,
    msg: UpdateProviderRequest,
) -> Result<UpdateProviderResponse, connectrpc::ConnectError> {
    let bytes = msg.encode_to_bytes();
    let view = UpdateProviderRequest::decode_view(&bytes).unwrap();
    match svc
        .update_provider(ctx, ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

async fn delete(
    svc: &ProviderService,
    ctx: RequestContext,
    msg: DeleteProviderRequest,
) -> Result<DeleteProviderResponse, connectrpc::ConnectError> {
    let bytes = msg.encode_to_bytes();
    let view = DeleteProviderRequest::decode_view(&bytes).unwrap();
    match svc
        .delete_provider(ctx, ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

fn provider_msg() -> CreateProviderRequest {
    CreateProviderRequest {
        kind: ProviderKind::Openai.into(),
        name: "deepseek".into(),
        // 故意带尾斜杠：顺带断言入库前被剪掉
        base_url: "https://api.example.com/v1/".into(),
        api_key: "sk-abcdef123456".into(),
        ..Default::default()
    }
}

// 记录假供应商收到的请求：路径 + 三种鉴权头
#[derive(Default)]
struct Hits {
    paths: Vec<String>,
    bearer: Vec<String>,
    x_api_key: Vec<String>,
    goog_key: Vec<String>,
}
type SharedHits = Arc<Mutex<Hits>>;

fn header_str(headers: &http::HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string()
}

// 单个 handler 打满所有路由：先记账，再按路径回不同形状
async fn fake_upstream(
    State(hits): State<SharedHits>,
    headers: http::HeaderMap,
    uri: http::Uri,
) -> Response {
    let path = uri.path().to_string();
    {
        let mut h = hits.lock().unwrap();
        h.paths.push(path.clone());
        h.bearer.push(
            headers
                .get(http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_string(),
        );
        h.x_api_key.push(header_str(&headers, "x-api-key"));
        h.goog_key.push(header_str(&headers, "x-goog-api-key"));
    }
    match path.as_str() {
        "/gemini" => Json(serde_json::json!({
            "models": [ { "name": "models/gemini-x" } ]
        }))
        .into_response(),
        "/fail" => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        _ => Json(serde_json::json!({ "data": [ { "id": "m-chat" } ] })).into_response(),
    }
}

// 起在 127.0.0.1 随机端口：每个测试独享一个假供应商，并行互不干扰
async fn spawn_fake() -> (String, SharedHits) {
    let hits: SharedHits = Arc::new(Mutex::new(Hits::default()));
    let app = Router::new()
        .route("/models", get(fake_upstream))
        .route("/custom-models", get(fake_upstream))
        .route("/gemini", get(fake_upstream))
        .route("/fail", get(fake_upstream))
        .with_state(hits.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), hits)
}

async fn fetch_models(
    svc: &ProviderService,
    ctx: RequestContext,
    msg: FetchModelsRequest,
) -> Result<FetchModelsResponse, connectrpc::ConnectError> {
    let bytes = msg.encode_to_bytes();
    let view = FetchModelsRequest::decode_view(&bytes).unwrap();
    match svc
        .fetch_models(ctx, ServiceRequest::from_parts(&view, &bytes))
        .await
    {
        Ok(resp) => Ok(owned_body(&resp.body)),
        Err(e) => Err(e),
    }
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn list_requires_bearer(pool: PgPool) {
    let svc = ProviderService::new(pool, SECRET, KEY_SECRET);
    let err = list(&svc, RequestContext::new(HeaderMap::new()))
        .await
        .err()
        .unwrap();
    assert_eq!(err.code, ErrorCode::Unauthenticated);
}

#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn list_empty_for_fresh_user(pool: PgPool) {
    let uid = new_user(&pool, "alice").await;
    let svc = ProviderService::new(pool, SECRET, KEY_SECRET);
    let r = list(&svc, auth_ctx(uid)).await.unwrap();
    assert!(r.providers.is_empty());
}

// 创建：响应里是脱敏值，库里是密文，尾斜杠被剪
#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn create_masks_response_and_seals_in_db(pool: PgPool) {
    let uid = new_user(&pool, "alice").await;
    let svc = ProviderService::new(pool.clone(), SECRET, KEY_SECRET);
    let r = create(&svc, auth_ctx(uid), provider_msg()).await.unwrap();
    let p = r.provider.as_option().unwrap();
    assert_eq!(p.api_key, "sk-****3456");
    assert_eq!(p.base_url, "https://api.example.com/v1");
    let row = store::list_by_user(&pool, uid).await.unwrap();
    let sealed = &row[0].api_key;
    assert_ne!(sealed, "sk-abcdef123456");
    let plain = lemma_crypto::open(&lemma_crypto::derive_key(KEY_SECRET), sealed).unwrap();
    assert_eq!(plain, "sk-abcdef123456");
}

// 创建：缺 name / 缺 base_url / 缺 api_key / 非法 kind 都是 InvalidArgument
#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn create_rejects_missing_fields(pool: PgPool) {
    let uid = new_user(&pool, "alice").await;
    let svc = ProviderService::new(pool, SECRET, KEY_SECRET);
    for bad in [
        CreateProviderRequest {
            name: "  ".into(),
            ..provider_msg()
        },
        CreateProviderRequest {
            base_url: "".into(),
            ..provider_msg()
        },
        CreateProviderRequest {
            api_key: "".into(),
            ..provider_msg()
        },
        CreateProviderRequest {
            kind: 999.into(),
            ..provider_msg()
        },
    ] {
        let err = create(&svc, auth_ctx(uid), bad).await.err().unwrap();
        assert_eq!(err.code, ErrorCode::InvalidArgument);
    }
}

// 更新：改名生效；api_key 传空串视为不变更（密文保持原样）
#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn update_renames_and_blank_key_keeps_sealed(pool: PgPool) {
    let uid = new_user(&pool, "alice").await;
    let svc = ProviderService::new(pool.clone(), SECRET, KEY_SECRET);
    let created = create(&svc, auth_ctx(uid), provider_msg()).await.unwrap();
    let id = created.provider.as_option().unwrap().id.clone();
    let sealed_before = store::list_by_user(&pool, uid).await.unwrap()[0]
        .api_key
        .clone();

    let r = update(
        &svc,
        auth_ctx(uid),
        UpdateProviderRequest {
            id,
            name: Some("renamed".into()),
            api_key: Some(String::new()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(r.provider.as_option().unwrap().name, "renamed");

    let sealed_after = store::list_by_user(&pool, uid).await.unwrap()[0]
        .api_key
        .clone();
    assert_eq!(sealed_before, sealed_after);
}

// 更新：动别人的 provider → NotFound；id 非法 → InvalidArgument
#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn update_rejects_other_owner_and_bad_id(pool: PgPool) {
    let u1 = new_user(&pool, "alice").await;
    let u2 = new_user(&pool, "erin").await;
    let svc = ProviderService::new(pool, SECRET, KEY_SECRET);
    let created = create(&svc, auth_ctx(u1), provider_msg()).await.unwrap();
    let id = created.provider.as_option().unwrap().id.clone();

    let err = update(
        &svc,
        auth_ctx(u2),
        UpdateProviderRequest {
            id: id.clone(),
            name: Some("hack".into()),
            ..Default::default()
        },
    )
    .await
    .err()
    .unwrap();
    assert_eq!(err.code, ErrorCode::NotFound);

    let err = update(
        &svc,
        auth_ctx(u1),
        UpdateProviderRequest {
            id: "not-a-uuid".into(),
            ..Default::default()
        },
    )
    .await
    .err()
    .unwrap();
    assert_eq!(err.code, ErrorCode::InvalidArgument);
}

// 删除：成功一次后再删 → NotFound
#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn delete_once_then_not_found(pool: PgPool) {
    let uid = new_user(&pool, "alice").await;
    let svc = ProviderService::new(pool, SECRET, KEY_SECRET);
    let created = create(&svc, auth_ctx(uid), provider_msg()).await.unwrap();
    let id = created.provider.as_option().unwrap().id.clone();

    delete(
        &svc,
        auth_ctx(uid),
        DeleteProviderRequest {
            id: id.clone(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let err = delete(
        &svc,
        auth_ctx(uid),
        DeleteProviderRequest {
            id,
            ..Default::default()
        },
    )
    .await
    .err()
    .unwrap();
    assert_eq!(err.code, ErrorCode::NotFound);
}

// 列表：正常行脱敏；密文损坏的行回退 "****"，不让一条脏数据炸掉整个接口
#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn list_masks_keys_and_garbage_falls_back(pool: PgPool) {
    let uid = new_user(&pool, "alice").await;
    let svc = ProviderService::new(pool.clone(), SECRET, KEY_SECRET);
    create(&svc, auth_ctx(uid), provider_msg()).await.unwrap();
    store::insert(
        &pool,
        &store::NewProvider {
            id: Uuid::new_v4(),
            user_id: uid,
            kind: "openai",
            name: "corrupt",
            base_url: "https://api.example.com/v1",
            api_key: "garbage-not-ciphertext",
            api_path: "",
            models_path: "",
            models: &[],
        },
    )
    .await
    .unwrap();

    let r = list(&svc, auth_ctx(uid)).await.unwrap();
    let mut rows: Vec<(String, String)> = r
        .providers
        .iter()
        .map(|p| (p.name.clone(), p.api_key.clone()))
        .collect();
    rows.sort();
    assert_eq!(
        rows,
        vec![
            ("corrupt".into(), "****".into()),
            ("deepseek".into(), "sk-****3456".into()),
        ]
    );
}

// 临时凭证模式：默认 /models 路径，明文 key 走 Bearer 头
#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn fetch_models_temp_credentials_openai(pool: PgPool) {
    let uid = new_user(&pool, "alice").await;
    let svc = ProviderService::new(pool, SECRET, KEY_SECRET);
    let (base, hits) = spawn_fake().await;
    let r = fetch_models(
        &svc,
        auth_ctx(uid),
        FetchModelsRequest {
            kind: ProviderKind::Openai.into(),
            base_url: base,
            api_key: "sk-tmp-123456".into(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(r.models, vec!["m-chat".to_string()]);
    let h = hits.lock().unwrap();
    assert_eq!(h.paths, vec!["/models".to_string()]);
    assert_eq!(h.bearer, vec!["Bearer sk-tmp-123456".to_string()]);
}

// 黄金场景：库里是密文，上游收到的是解密后的明文（anthropic 走 x-api-key）
#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn fetch_models_saved_provider_decrypts_key(pool: PgPool) {
    let uid = new_user(&pool, "alice").await;
    let svc = ProviderService::new(pool.clone(), SECRET, KEY_SECRET);
    let (base, hits) = spawn_fake().await;
    let created = create(
        &svc,
        auth_ctx(uid),
        CreateProviderRequest {
            kind: ProviderKind::Anthropic.into(),
            name: "fake".into(),
            base_url: base,
            api_key: "sk-anthropic-987654321".into(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let id = created.provider.as_option().unwrap().id.clone();

    let r = fetch_models(
        &svc,
        auth_ctx(uid),
        FetchModelsRequest {
            id,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(r.models, vec!["m-chat".to_string()]);
    let h = hits.lock().unwrap();
    assert_eq!(h.x_api_key, vec!["sk-anthropic-987654321".to_string()]);
}

// gemini：models/ 前缀剥离 + 自定义 models_path 生效 + x-goog-api-key 头
#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn fetch_models_gemini_strips_prefix_and_custom_path(pool: PgPool) {
    let uid = new_user(&pool, "alice").await;
    let svc = ProviderService::new(pool, SECRET, KEY_SECRET);
    let (base, hits) = spawn_fake().await;
    let r = fetch_models(
        &svc,
        auth_ctx(uid),
        FetchModelsRequest {
            kind: ProviderKind::Gemini.into(),
            base_url: base,
            api_key: "goog-key-123456".into(),
            models_path: "/gemini".into(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(r.models, vec!["gemini-x".to_string()]);
    let h = hits.lock().unwrap();
    assert_eq!(h.paths, vec!["/gemini".to_string()]);
    assert_eq!(h.goog_key, vec!["goog-key-123456".to_string()]);
}

// 错误映射：上游 500 → internal；非法 kind → InvalidArgument；不存在的 id → NotFound
#[sqlx::test(migrations = "../lemma-db/migrations")]
async fn fetch_models_maps_errors_and_bad_id(pool: PgPool) {
    let uid = new_user(&pool, "alice").await;
    let svc = ProviderService::new(pool, SECRET, KEY_SECRET);
    let (base, _hits) = spawn_fake().await;

    let err = fetch_models(
        &svc,
        auth_ctx(uid),
        FetchModelsRequest {
            kind: ProviderKind::Openai.into(),
            base_url: format!("{base}/"),
            api_key: "k".into(),
            models_path: "/fail".into(),
            ..Default::default()
        },
    )
    .await
    .err()
    .unwrap();
    assert_eq!(err.code, ErrorCode::Internal);

    let err = fetch_models(
        &svc,
        auth_ctx(uid),
        FetchModelsRequest {
            kind: 999.into(),
            base_url: base,
            api_key: "k".into(),
            ..Default::default()
        },
    )
    .await
    .err()
    .unwrap();
    assert_eq!(err.code, ErrorCode::InvalidArgument);

    let err = fetch_models(
        &svc,
        auth_ctx(uid),
        FetchModelsRequest {
            id: Uuid::new_v4().to_string(),
            ..Default::default()
        },
    )
    .await
    .err()
    .unwrap();
    assert_eq!(err.code, ErrorCode::NotFound);
}
