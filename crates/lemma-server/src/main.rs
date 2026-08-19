mod config;

use std::sync::Arc;

use lemma_auth::AuthService;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config = config::Config::from_env();

    let pool = lemma_db::connect(&config.database_url)
        .await
        .expect("database connect");
    lemma_db::migrate(&pool).await.expect("database migrate");

    let auth = Arc::new(AuthService::new(pool, config.jwt_secret));
    let connect = connectrpc::Router::new().add_service(auth);

    let app = axum::Router::new().fallback_service(connect.into_axum_service());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1025").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
