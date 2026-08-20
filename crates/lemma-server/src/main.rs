mod config;

use lemma_auth::AuthService;
use lemma_conversations::ConversationService;
use lemma_providers::ProviderService;

use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let config = config::Config::from_env()?;

    let pool = lemma_db::connect(&config.database_url).await?;
    lemma_db::migrate(&pool).await?;

    let auth = Arc::new(AuthService::new(pool.clone(), config.jwt_secret.clone()));
    let provider = Arc::new(ProviderService::new(
        pool.clone(),
        config.jwt_secret.clone(),
        config.secret_key,
    ));
    let conversations = Arc::new(ConversationService::new(pool, config.jwt_secret));
    let connect = connectrpc::Router::new()
        .add_service(auth)
        .add_service(provider)
        .add_service(conversations);

    let app = axum::Router::new().fallback_service(connect.into_axum_service());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1025").await?;
    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
