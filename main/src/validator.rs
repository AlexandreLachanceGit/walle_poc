use ed25519_dalek::{PublicKey, Signature, Verifier};
use http::HeaderMap;
use lambda_http::Body;

pub fn validate_discord_signature(headers: &HeaderMap, body: &Body) -> anyhow::Result<()> {
    let pub_key: PublicKey = PublicKey::from_bytes(
        &hex::decode(
            std::env::var("DISCORD_PUBLIC_KEY")
                .expect("Expected DISCORD_PUBLIC_KEY to be set in the environment"),
        )
        .expect("Couldn't hex::decode the DISCORD_PUBLIC_KEY"),
    )
    .expect("Couldn't create a PublicKey from DISCORD_PUBLIC_KEY bytes");

    let sig_ed25519 = {
        let header_signature = headers
            .get("X-Signature-Ed25519")
            .ok_or(anyhow::anyhow!("missing X-Signature-Ed25519 header"))?;
        let decoded_header = hex::decode(header_signature)?;

        let mut sig_arr: [u8; 64] = [0; 64];
        for (i, byte) in decoded_header.into_iter().enumerate() {
            sig_arr[i] = byte;
        }
        Signature::from_bytes(&sig_arr)
    }
    .unwrap();
    let sig_timestamp = headers
        .get("X-Signature-Timestamp")
        .ok_or(anyhow::anyhow!("missing X-Signature-Timestamp header"))?;

    if let Body::Text(body) = body {
        let content = sig_timestamp
            .as_bytes()
            .iter()
            .chain(body.as_bytes().iter())
            .cloned()
            .collect::<Vec<u8>>();

        pub_key
            .verify(&content.as_slice(), &sig_ed25519)
            .map_err(anyhow::Error::msg)
    } else {
        Err(anyhow::anyhow!("Invalid body type"))
    }
}
