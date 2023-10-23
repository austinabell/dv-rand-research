mod bls;

use axum::body::Bytes;
use futures::{stream, StreamExt};
use rand::{rngs::StdRng, Rng, SeedableRng};
use sha2::{Digest, Sha256};
use tracing::info;

const CONCURRENT_REQUESTS: usize = 4;

#[derive(Debug)]
struct NodeClient {
    address: &'static str,
    pub_key: Bytes,
}

fn xor_randomness(current_randomness: &mut [u8; 96], new_randomness: &Bytes) {
    // TODO verify new randomness is of correct length
    for (a, b) in current_randomness.iter_mut().zip(new_randomness.iter()) {
        *a ^= b;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let client = reqwest::Client::new();
    // The list of node URLs
    // TODO have this be specified with an environment variable
    let nodes = ["http://localhost:3001", "http://localhost:3002"];

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
    let mut current_randomness = [0u8; 96];
    let mut nonce: u32 = 0;

    loop {
        // Select a node based on some function of the current randomness
        // NOTE: This is only verifiable if the nonce is available and synchronized between nodes.
        // 		 This mimics what height would do for leader election in a network.
        let mut hasher = Sha256::new();
        hasher.update(current_randomness);
        hasher.update(nonce.to_le_bytes());
        let seed: [u8; 32] = hasher.finalize().into();

        let mut rng = StdRng::from_seed(seed);
        let random_value: usize = rng.gen();
        let node_idx = random_value % nodes.len();
        let selected_node = &nodes[node_idx];

        // Increment nonce to ensure that failed request don't send to the same node continually
        nonce = nonce.wrapping_add(1);

        // TODO refactor this out
        // HTTP request to the node
        let res = match client
            .post(selected_node.address.to_string() + "/sign")
            .body(current_randomness.to_vec())
            .send()
            .await
        {
            Ok(res) => res,
            Err(e) => {
                tracing::warn!(
                    "Error sending request to {}: {:?}",
                    selected_node.address,
                    e
                );
                continue;
            }
        };

        // Read bytes from response body
        let rand_bytes: Bytes = match res.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::warn!(
                    "Error sending request to {}: {:?}",
                    selected_node.address,
                    e
                );
                continue;
            }
        };

        bls::verify_randomness_bytes(&rand_bytes, &selected_node.pub_key, &current_randomness)?;

        // Update the current randomness by xor retrieved with previous signature
        xor_randomness(&mut current_randomness, &rand_bytes);

        // Log the randomness and selected node
        info!(
            "Randomness updated: {}",
            bs58::encode(&current_randomness).into_string()
        );

        // Sleep time in between requests to simulate block/round timings
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
