mod bls;

use bls::SecretKey;

use axum::body::Bytes;
use axum::extract::State;
use axum::routing::{get, post};
use axum::Router;
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO setup logging

    let port = env::var("NODE_PORT").unwrap_or_else(|_| "3000".to_string());

    let state = Arc::new(bls::random_test_key());

    // Build our application with a route
    let app = Router::new()
        .route("/sign", post(handle_randomness))
        .route("/public-key", get(get_public_key))
        .with_state(state.clone());

    // Run it with hyper
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn handle_randomness(State(sk): State<Arc<SecretKey>>, body: Bytes) -> Result<Bytes, String> {
    // Arbitrarily sign the message sent in
    // NOTE: This is a terrible idea to do in any practical use case, but just to assume node is
    // following and validating what is being signed.
    bls::sign_randomness(&sk, &body)
}

async fn get_public_key(State(sk): State<Arc<SecretKey>>) -> Bytes {
    // Return public key of server for verification of signatures
    Bytes::copy_from_slice(&sk.sk_to_pk().to_bytes())
}
