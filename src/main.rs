use anyhow::Context;
use base64::URL_SAFE_NO_PAD;
use clap::Clap;
use openssl::{ec::EcKey, ecdsa::EcdsaSig, hash::MessageDigest, nid::Nid, pkey::Private};
use serde_json::{json, Value};
use std::path::PathBuf;
use uuid::Uuid;

/// A small command line interface to sign POST requests for Payouts/Paydirect API.
#[derive(Clap)]
struct Command {
    /// The payload you want to sign.
    #[clap(long)]
    body: String,
    /// The filename of the Elliptic Curve private key used to sign, in PEM format.
    #[clap(long)]
    key: PathBuf,
    /// The certificate id associated to the public certificate you uploaded in TrueLayer's Console.
    /// The certificate id can be retrieved in the Payouts Setting section.
    /// It will be used as the `kid` header in the JWS.
    #[clap(long)]
    kid: Uuid,
}

impl Command {
    /// Parse the EC private key from the specified file.
    pub fn private_key(&self) -> anyhow::Result<EcKey<Private>> {
        let raw_private_key =
            std::fs::read(&self.key).context("Failed to read the private key file.")?;
        let private_key = openssl::pkey::PKey::private_key_from_pem(&raw_private_key)
            .context("Failed to parse the private key as PEM.")?
            .ec_key()
            .context("The private key must be an Elliptic Curve key.")?;
        private_key.check_key().context("Key verification failed")?;
        Ok(private_key)
    }
}

#[derive(serde::Serialize)]
pub struct JwsPayload {
    #[serde(rename = "Content-Type")]
    content_type: String,
    body: Value,
}

pub fn main() -> anyhow::Result<()> {
    let options = Command::parse();

    let jws_header = json!({
        "alg": "ES512",
        "kid": options.kid.to_string()
    });
    let private_key = options.private_key()?;
    // println!("Request payload:\n{}\n", &jws_payload);

    let jws = get_jws(&jws_header, options.body.as_bytes(), private_key)?;
    // println!("JWS:\n{}\n", jws);

    let parts = jws.split(".").collect::<Vec<_>>();
    let detached_jsw = format!("{}..{}", parts[0], parts[2]);
    // Omit the payload for a JWS with detached payload
    println!("{}", detached_jsw);

    Ok(())
}

/// Get a JWS using the ES512 signing scheme.
///
/// Check section A.4 of RFC7515 for the details: https://www.rfc-editor.org/rfc/rfc7515.txt
pub fn get_jws(
    jws_header: &Value,
    jws_payload: &[u8],
    pkey: EcKey<Private>,
) -> Result<String, anyhow::Error> {
    let to_be_signed = format!(
        "{}.{}",
        base64_encode(serde_json::to_string(&jws_header)?.as_bytes()),
        base64_encode(jws_payload),
    );
    let signature = sign_es512(to_be_signed.as_bytes(), pkey)?;

    let jws = format!(
        "{}.{}.{}",
        base64_encode(serde_json::to_string(&jws_header)?.as_bytes()),
        base64_encode(jws_payload),
        signature
    );
    Ok(jws)
}

/// Sign a payload using the provided private key and return the signature as a base64 encoded string.
///
/// Check section A.4 of RFC7515 for the details: https://www.rfc-editor.org/rfc/rfc7515.txt
pub fn sign_es512(payload: &[u8], pkey: EcKey<Private>) -> Result<String, anyhow::Error> {
    if pkey.group().curve_name() != Some(Nid::SECP521R1) {
        return Err(anyhow::anyhow!(
            "The underlying elliptic curve must be P-521 to sign using ES512."
        ));
    }
    let hash = openssl::hash::hash(MessageDigest::sha512(), &payload)?;
    let structured_signature = EcdsaSig::sign(&hash, &pkey)?;

    let r = structured_signature.r().to_vec();
    let s = structured_signature.s().to_vec();
    let mut signature_bytes: Vec<u8> = Vec::new();
    // Padding to fixed length
    signature_bytes.extend(std::iter::repeat(0x00).take(66 - r.len()));
    signature_bytes.extend(r);
    // Padding to fixed length
    signature_bytes.extend(std::iter::repeat(0x00).take(66 - s.len()));
    signature_bytes.extend(s);

    Ok(base64_encode(&signature_bytes))
}

/// Base64 encoding according to RFC7515 - see `Base64url` in section 2.
pub fn base64_encode(payload: &[u8]) -> String {
    base64::encode_config(payload, URL_SAFE_NO_PAD)
}
