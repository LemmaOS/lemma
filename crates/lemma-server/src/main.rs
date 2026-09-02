//! The Lemma server binary: wires the domain services into a Connect
//! router and serves the embedded web build.

mod config;
mod web;

use std::sync::Arc;

use lemma_auth::AuthService;
use lemma_chat::ChatService;
use lemma_chat::adapter::{DispatchAdapter, LlmAdapter};
use lemma_conversations::ConversationService;
use lemma_providers::ProviderService;
use lemma_sync::SyncService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let config = config::Config::from_env()?;

    let pool = lemma_db::connect(&config.database_url).await?;
    lemma_db::migrate(&pool).await?;

    let adapter: Arc<dyn LlmAdapter> = Arc::new(DispatchAdapter::new());
    let auth = Arc::new(AuthService::new(pool.clone(), config.jwt_secret.clone()));
    let provider = Arc::new(ProviderService::new(
        pool.clone(),
        config.jwt_secret.clone(),
        config.secret_key.clone(),
    ));
    let conversations = Arc::new(ConversationService::new(
        pool.clone(),
        config.jwt_secret.clone(),
        lemma_archive::DbArchiveSource::new(pool.clone(), config.secret_key.clone()),
    ));
    let storage = Arc::new(lemma_archive::StorageService::new(
        pool.clone(),
        config.jwt_secret.clone(),
        config.secret_key.clone(),
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
        .add_service(storage)
        .add_service(conversations)
        .add_service(chat)
        .add_service(sync);

    let connect_service = connect.into_axum_service();
    let mut app = axum::Router::new().fallback(web::handler);
    // Connect RPC paths are /<package>.<Service>/<Method>; each service
    // gets an explicit prefix route, and everything else falls through
    // to the web app.
    for svc in [
        "AuthService",
        "ProviderService",
        "StorageService",
        "ConversationService",
        "ChatService",
        "SyncService",
    ] {
        app = app.route_service(
            &format!("/lemma.v1.{svc}/{{*path}}"),
            connect_service.clone(),
        );
    }
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1025").await?;
    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
