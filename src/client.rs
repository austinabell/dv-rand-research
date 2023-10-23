use axum::body::Bytes;
use rand::{rngs::StdRng, Rng, SeedableRng};
use sha2::{Digest, Sha256};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // The list of node URLs
    // TODO have this be specified with an environment variable
    let nodes = ["http://localhost:3001", "http://localhost:3002"];
    // TODO query for all public keys to verify signatures

    // Initialize the current randomness as all zeroes
    let mut current_randomness: Bytes = Bytes::from(&[0u8; 32][..]);

    loop {
        // Select a node based on some function of the current randomness
        let array: [u8; 32] = Sha256::digest(&current_randomness).into();

        let mut rng = StdRng::from_seed(array);
        let random_value: usize = rng.gen();
        let node_idx = random_value % nodes.len();
        let node_url = nodes[node_idx];

        // Perform HTTP request to the node
        let client = reqwest::Client::new();
        let res = client
            .post(node_url.to_string() + "/sign")
            .body(current_randomness)
            .send()
            // TODO gracefully handle errors
            .await?;

        // Assume the node returns a new random hash as a hex string
        let new_random_hash: Bytes = res.bytes().await?;

        // Update the current randomness
        current_randomness = new_random_hash;

        // Log the randomness and selected node
        println!(
            "Current randomness: {}",
            bs58::encode(&current_randomness).into_string()
        );
        println!("Selected node: {}", node_url);

        // Simulating continuous operation
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
