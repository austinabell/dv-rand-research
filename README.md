# Distributed Verifiable Randomness

## Goals

- Publicly verifiable
- Output of VRF is unpredictable
- Distributed generation of randomness
  - Either through threshold signatures or some election of randomness generation

## Why a VRF?

- Randomness for leader election protocols
- Randomness oracle to use in computation
- Sharding data in a network in unbiased way
- Randomly choosing and matching peers in a network to gossip with
- Verifiable randomness for commitments in zero-knowledge proofs
- Private nonce generation to increase security of signatures like schnorr, making reverse-engineering signatures infeasible

## Use of randomness in protocols

- Ethereum (Beacon chain): RANDAO like randomness with BLS signature that gets `xor`ed with the previous seed
  - https://github.com/ethereum/annotated-spec/blob/master/phase0/beacon-chain.md#aside-randao-seeds-and-committee-generation
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

<!-- ### Native VRF with election

### Threshold signatures -->

## Installation

<!-- TODO -->

## Usage

<!-- TODO -->
