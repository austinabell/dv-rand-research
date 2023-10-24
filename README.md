# Distributed Verifiable Randomness

## Goals

- Publicly verifiable
- Output of VRF is unpredictable
- Distributed generation of randomness
  - Either through threshold signatures or some election of randomness generation

## Why distributed randomness?

- Randomness for leader election protocols
- Randomness oracle to use in computation
- Sharding data in a network in unbiased way
- Randomly choosing and matching peers in a network to gossip with
- Verifiable randomness for commitments in zero-knowledge proofs
- Private nonce generation to increase security of signatures like schnorr, making reverse-engineering signatures infeasible

## Examples of randomness for leader elections

- Ethereum (Beacon chain): RANDAO like randomness with BLS signature that gets `xor`ed with the previous seed
  - [spec](https://github.com/ethereum/annotated-spec/blob/master/phase0/beacon-chain.md#aside-randao-seeds-and-committee-generation) and [book](https://eth2book.info/capella/part2/building_blocks/randomness/) links
  - One bit of manipulation power (produce or not produce block and sac reward)
  - BLS signatures deterministic/unique signing means it can't be gamed/grinded
- NEAR: Block producers generate VRF based on 25519 generated using the previous block's hash of the vrf value
  - Random seed used in transactions is the hash of that vrf value
  - Leader rotation is based on this randomness (weighted by stake)
  - Seed for rotation is based on epoch, so the rotation is predictable within epoch
- Filecoin: Uses [drand randomness beacon](https://drand.love)
  - Nodes fetch data from drand, use with other chain data to create ticket
  - Miners run VDF on the ticket, this determines if they are one of the producers of a tipset
    - VDF for time-locking to make it harder to game system and seal the randomness
  - Drand round have fixed interval based on [NTP](https://en.wikipedia.org/wiki/Network_Time_Protocol)
- Algorand: VRF to choose committee and leader election
  - Forked libsodium for vrf implementation 
  - Weighted selection by stake

## Options

### Chained vs Unchained

- Chained: Hash link with previous
  - Less predictable randomness can be good, but also might not be ideal for leader elections where validators need to prepare for transactions they will be validating
  - Requires data availability of all rounds in the chain to produce another
- Unchained: Message is just based on some predictable value (round, block, epoch)
  - Can predict message ahead of time, enables timelock encryption (decryption after specified time period)
    - Drand does timelocks through swapping G1 (pk) and G2 (sig) to be able to aggregate public key instead of signature, signatures also 50% smaller (48 bytes) as a result. Use group public key for encryption

### VDF

- Minimizes biasability because calculating outcome combinations would be computationally infeasible and unusable outside of very long tail of consecutive elections of the same party
  - Concerns of usage in practice due to attack vectors and algo not being tested enough or understood to use https://ethresear.ch/t/statement-regarding-the-public-report-on-the-analysis-of-minroot/16670

### Single Secret Leader Election

- Increase unpredictability of leader elections through some primitive like [threshold FHE](https://eprint.iacr.org/2020/025) or [size-2 blind-and-swap](https://ethresear.ch/t/simplified-ssle/12315)
  - Not going down this path now because threshold FHE isn't implemented (estimate 2024) and other option is theoretical and hasn't been tested in practice
  - If biasability of the alternative solution is concern, this can be explored

## Choices in PoC code

Overall choices were made to be a very simplified approach to how randomness for leader elections would be.

- Chained randomness: Randomness is mixed with previous randomness, because assuming a single "block producer", randomness would be easily predictable if not
- BLS signatures: Chosen just to closely match Eth2, library maturity, and to be able to experiment with swapping G1 and G2
  - Kept signatures in G2 with public key in G1 because in practice we would want to keep the public keys stored as minimally sized as possible. Also allows for aggregating the signatures and provides more bytes for the randomness output
  - Signature used as dual purpose for random output and proof where a VRF would have those be separate
- Leader (node) chosen to provide randomness to "randao" state by hashing the current state and an increasing nonce and selecting an index modulo the number of nodes
  - Nonce in this case mimics a block height to rotate the leader in case of an error or no response from the node to avoid network from stalling
- `client` bin in this crate just simulates what consensus would agree on. Nodes in the PoC are not synced over a p2p network for simplicity and just serve data over a REST API.

### Future Options

- Switch nodes to synchronizing over p2p network, maybe select rounds based on NTP time like drand and do leader election based on this
- Compare and benchmark this approach with a threshold aggregated BLS signature from all validators
  - Would be more resilient, less biasability of randomness, but slower to generate and verify
  - Could also compare against VRF schemes
- Experiment with more complex latency, dropped packets, malicious nodes
  - Measure cost of grinding (provide/abstain combinations)
  - Explore VDF to counteract above
- Weighted leader selection
  - Simulate stake weighted selection

## Usage

Start test network with:

```
docker compose up
```

Or run each independently for a specific configuration:

```
# Node
NODE_PORT=<PORT> cargo run --bin node

# Client
NODE_ADDRESSES="http://localhost:3000,http://localhost:3001" cargo run --bin client
```
