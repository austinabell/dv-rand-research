mod bls;
use bls::RandState;

mod utils;

use axum::body::Bytes;
use futures::{stream, StreamExt};
use rand::{rngs::StdRng, Rng, SeedableRng};
use reqwest::Client;
use sha2::{Digest, Sha256};
use tracing::info;

const CONCURRENT_REQUESTS: usize = 4;

#[derive(Debug)]
struct NodeClient {
    address: String,
    pub_key: Bytes,
}

/// Mix the new randomness with the current randomness by performing a xor operation
fn xor_randomness(current_randomness: &mut RandState, new_randomness: &Bytes) {
    // Sanity check for randomness length. This should be verified by the signature verification.
    debug_assert_eq!(current_randomness.len(), new_randomness.len());

    for (a, b) in current_randomness.iter_mut().zip(new_randomness.iter()) {
        *a ^= b;
    }
}

/// Query and verify randomness
async fn query_update_randomness(
    client: &Client,
    node: &NodeClient,
    randomness: &mut RandState,
) -> anyhow::Result<()> {
    let res = client
        .post(node.address.to_string() + "/sign")
        .body(randomness.to_vec())
        .send()
        .await?;

    // Read bytes from response body
    let rand_bytes: Bytes = res.bytes().await?;

    // Verify signature against node's public key before updating state.
    bls::verify_randomness_bytes(&rand_bytes, &node.pub_key, randomness)?;

    // Update the current randomness by xor retrieved with previous signature
    xor_randomness(randomness, &rand_bytes);

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let client = reqwest::Client::new();

    // Read the node addresses from NODE_ADDRESSES environment variable
    let node_addresses = std::env::var("NODE_ADDRESSES").expect("must provide NODE_ADDRESSES");
    let nodes: Vec<String> = node_addresses.split(',').map(From::from).collect();

    let bodies = stream::iter(nodes)
        .map(|address| {
            let client = &client;
            async move {
                let res = client
                    .get(address.to_string() + "/public-key")
                    .send()
                    .await
                    .unwrap();
                NodeClient {
                    address,
                    pub_key: res.bytes().await.unwrap(),
                }
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    let nodes: Vec<NodeClient> = bodies.collect().await;

    // Initialize the current randomness as all zeroes
    let mut rand_state = [0u8; 96];
    let mut nonce: u32 = 0;

    loop {
        // Select a node based on some function of the current randomness
        // NOTE: This is only verifiable if the nonce is available and synchronized between nodes.
        // 		 This mimics what height would do for leader election in a network.
        //       Also, the random selection would not be like this, but is done for simplicity.
        let mut hasher = Sha256::new();
        hasher.update(rand_state);
        hasher.update(nonce.to_le_bytes());
        let seed: [u8; 32] = hasher.finalize().into();

        let mut rng = StdRng::from_seed(seed);
        let random_value: usize = rng.gen();
        let node_idx = random_value % nodes.len();
        let selected_node = &nodes[node_idx];

        // Increment nonce to ensure that failed request don't send to the same node continually
        nonce = nonce.wrapping_add(1);

        // HTTP request to the node
        if let Err(e) = query_update_randomness(&client, selected_node, &mut rand_state).await {
            tracing::warn!(
                "Error sending request to {}: {:?}",
                selected_node.address,
                e
            );
            continue;
        }

        // Log the randomness and selected node
        info!(
            "Randomness updated: {}",
            utils::short_bytes_format(&rand_state)
        );

        // Sleep time in between requests to simulate block/round timings
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
