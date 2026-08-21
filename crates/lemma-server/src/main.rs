mod config;

use std::sync::Arc;

use lemma_auth::AuthService;
use lemma_chat::ChatService;
use lemma_chat::adapter::{LlmAdapter, OpenAiCompatible};
use lemma_conversations::ConversationService;
use lemma_providers::ProviderService;
use lemma_sync::SyncService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let config = config::Config::from_env()?;

    let pool = lemma_db::connect(&config.database_url).await?;
    lemma_db::migrate(&pool).await?;

    // 适配器以 trait object 注入，测试可换假实现
    let adapter: Arc<dyn LlmAdapter> = Arc::new(OpenAiCompatible::new());
    let auth = Arc::new(AuthService::new(pool.clone(), config.jwt_secret.clone()));
    let provider = Arc::new(ProviderService::new(
        pool.clone(),
        config.jwt_secret.clone(),
        config.secret_key.clone(),
    ));
    let conversations = Arc::new(ConversationService::new(
        pool.clone(),
        config.jwt_secret.clone(),
    ));
    let chat = Arc::new(ChatService::new(
        pool.clone(),
        config.jwt_secret.clone(),
        config.secret_key,
        adapter,
    ));
    let sync = Arc::new(SyncService::new(pool, config.jwt_secret));
    let connect = connectrpc::Router::new()
        .add_service(auth)
        .add_service(provider)
        .add_service(conversations)
        .add_service(chat)
        .add_service(sync);

    let app = axum::Router::new().fallback_service(connect.into_axum_service());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1025").await?;
    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
