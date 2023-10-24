pub(crate) fn short_bytes_format(bytes: impl AsRef<[u8]>) -> String {
    let encoded = bs58::encode(bytes).into_string();

    if encoded.len() <= 10 {
        encoded
    } else {
        let prefix = &encoded[0..5];
        let suffix = &encoded[encoded.len() - 5..];
        format!("{}...{}", prefix, suffix)
    }
}
