#![allow(unused)]

use anyhow::{anyhow, bail};
use axum::body::Bytes;
use blst_core::{PublicKey, Signature};
use rand::RngCore;

use blst::min_pk as blst_core;

const RAND_LEN: usize = 96;

pub(crate) type RandState = [u8; RAND_LEN];
pub(crate) type SecretKey = blst_core::SecretKey;

// Domain separation tag for BLS signatures.
const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

pub(crate) fn random_test_key() -> SecretKey {
    // Generate in-memory BLS key
    let mut rng = rand::thread_rng();
    let mut ikm = [0u8; 32];
    rng.fill_bytes(&mut ikm);

    SecretKey::key_gen(&ikm, &[]).unwrap()
}

pub(crate) fn sign_randomness(sk: &SecretKey, data: &Bytes) -> Result<Bytes, String> {
    if data.len() != RAND_LEN {
        return Err(format!("Randomness must be 96 bytes, was {}", data.len()));
    }

    let sig: Signature = sk.sign(data, DST, &[]);

    // Serialize the signature into bytes (compressed)
    Ok(Bytes::copy_from_slice(&sig.to_bytes()))
}

pub(crate) fn verify_randomness_bytes(
    rand: &Bytes,
    pub_key: &Bytes,
    msg: &RandState,
) -> anyhow::Result<()> {
    let sig = Signature::from_bytes(rand)
        .map_err(|e| anyhow!("failed to deserialize signature: {:?}", e))?;
    let pk = PublicKey::from_bytes(pub_key)
        .map_err(|e| anyhow!("failed to deserialize public key: {:?}", e))?;

    let err = sig.verify(true, msg, DST, &[], &pk, true);
    if err != blst::BLST_ERROR::BLST_SUCCESS {
        bail!("Signature verification failed with code: {:?}", err)
    }

    Ok(())
}
