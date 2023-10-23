use axum::body::Bytes;
use axum::extract::State;
use axum::routing::{get, post};
use axum::Router;
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

async fn handle_randomness(State(sk): State<Arc<SecretKey>>, body: Bytes) -> Bytes {
	// TODO verify body is of correct length (96 bytes)
    // Arbitrarily sign the message sent in
    // NOTE: This is a terrible idea to do in any practical use case, but just to assume node is
    // following and validating what is being signed.
    let sig: Signature = sk.sign(&body, DST, &[]);

    // Serialize the signature into bytes (compressed)
    let sig_bytes = Bytes::copy_from_slice(&sig.to_bytes());
    println!("signature {}", bs58::encode(&sig_bytes).into_string());

    // Return the encoded result
    sig_bytes
}

async fn get_public_key(State(sk): State<Arc<SecretKey>>) -> Bytes {
    // Return public key of server for verification of signatures
    Bytes::copy_from_slice(&sk.sk_to_pk().to_bytes())
}
