use axum::body::Bytes;
use rand::{rngs::StdRng, Rng, SeedableRng};
use sha2::{Digest, Sha256};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The list of node URLs
    let nodes = vec!["http://localhost:3001", "http://localhost:3002"];

    // Initialize the current randomness as all zeros
    let mut current_randomness: Bytes = Bytes::from(&[0u8; 32][..]);

    loop {
        // Select a node based on some function of the current randomness
        // For simplicity, let's just use the first byte to index into the nodes array
        let array: [u8; 32] = Sha256::digest(&current_randomness).into();

        let mut rng = StdRng::from_seed(array);
        let random_value: usize = rng.gen();
        let node_idx = random_value % nodes.len();
        let node_url = nodes[node_idx];

        // Perform HTTP request to the node
        let client = reqwest::Client::new();
        let res = client
            .post(node_url)
            .body(current_randomness)
            .send()
            // TODO gracefully handle errors
            .await?;

        // Assume the node returns a new random hash as a hex string
        let new_random_hash: Bytes = res.bytes().await?;

        // Update the current randomness
        current_randomness = new_random_hash;

        // Log the randomness and selected node
        println!("Current randomness: {:?}", hex::encode(&current_randomness));
        println!("Selected node: {}", node_url);

        // Simulating continuous operation
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
