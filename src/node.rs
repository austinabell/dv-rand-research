use axum::body::Bytes;
use axum::extract::State;
use axum::{routing::post, Router};
// TODO benchmark this against min_sig
use blst::min_pk::{SecretKey, Signature};
use rand::RngCore;
use std::env;
use std::sync::Arc;

// TODO look into this and if it's necessary
const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // TODO setup logging

    let port = env::var("NODE_PORT").unwrap_or_else(|_| "3000".to_string());

    // Generate in-memory BLS key
    let mut rng = rand::thread_rng();
    let mut ikm = [0u8; 32];
    rng.fill_bytes(&mut ikm);

    let sk = SecretKey::key_gen(&ikm, &[]).unwrap();
    let state = Arc::new(sk);

    // Build our application with a route
    let app = Router::new()
        .route("/", post(handle_randomness))
        .with_state(state.clone());

    // Run it with hyper
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn handle_randomness(State(sk): State<Arc<SecretKey>>, body: Bytes) -> Bytes {
    // Arbitrarily sign the message sent in
    // NOTE: This is a terrible idea to do in any practical use case, but just to assume node is
    // following and validating what is being signed.
    let sig: Signature = sk.sign(&body, DST, &[]);

    println!("{sig:?}");
    // Return the encoded result
    Bytes::copy_from_slice(&sig.to_bytes())
}
