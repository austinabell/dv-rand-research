use anyhow::{anyhow, bail};
use axum::body::Bytes;
use blst::min_pk::Signature;

pub(crate) type RandOutput = [u8; 96];
pub(crate) type PublicKey = blst::min_pk::PublicKey;

// TODO look into this and if it's necessary
pub(crate) const DST: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";

pub(crate) fn verify_randomness_bytes(
    rand: &Bytes,
    pub_key: &Bytes,
    msg: &RandOutput,
) -> anyhow::Result<()> {
    let sig = Signature::from_bytes(&rand)
        .map_err(|e| anyhow!("failed to deserialize signature: {:?}", e))?;
    let pk = PublicKey::from_bytes(&pub_key)
        .map_err(|e| anyhow!("failed to deserialize public key: {:?}", e))?;

    let err = sig.verify(true, msg, DST, &[], &pk, true);
    if err != blst::BLST_ERROR::BLST_SUCCESS {
        bail!("Signature verification failed with code: {:?}", err)
    }

    Ok(())
}
