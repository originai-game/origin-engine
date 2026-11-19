//! WebTransport's framing, shared by the server, the native client and the
//! load bots: one bidirectional stream the client opens carries every
//! message but the datagrams ([`crate::protocol::client_datagram`],
//! [`crate::protocol::server_datagram`]), each as its length (u32 LE) and
//! its bytes.

use crate::protocol::MAX_MESSAGE;

/// One message as a stream frame.
pub fn frame(bytes: &[u8]) -> Vec<u8> {
    let mut f = Vec::with_capacity(4 + bytes.len());
    f.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    f.extend_from_slice(bytes);
    f
}

/// The next frame of a stream; an error once it ends or breaks.
pub async fn read_frame(recv: &mut web_transport_quinn::RecvStream) -> Result<Vec<u8>, String> {
    let mut len = [0u8; 4];
    recv.read_exact(&mut len).await.map_err(|e| e.to_string())?;
    let len = u32::from_le_bytes(len) as usize;
    // A server's messages may be larger than a client's (MAX_MESSAGE): an enter burst is several.
    if len > 4 * MAX_MESSAGE {
        return Err(format!("a {len}-byte frame is too long"));
    }
    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf).await.map_err(|e| e.to_string())?;
    Ok(buf)
}

/// A WebTransport session to `url` (`https://host:port/`): the server's
/// certificate is checked against `cert_hashes` (SHA-256, as a browser's
/// `serverCertificateHashes`) when any are given, else against the
/// system's roots.
pub async fn dial(
    url: &str,
    cert_hashes: &[[u8; 32]],
) -> Result<web_transport_quinn::Session, String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("'{url}': {e}"))?;
    if parsed.scheme() != "https" {
        return Err(format!(
            "'{url}' is not a WebTransport URL: write https://host:port/"
        ));
    }
    let builder = web_transport_quinn::ClientBuilder::new();
    let client = if cert_hashes.is_empty() {
        builder.with_system_roots()
    } else {
        builder.with_server_certificate_hashes(cert_hashes.iter().map(|h| h.to_vec()).collect())
    }
    .map_err(|e| e.to_string())?;
    client.connect(parsed).await.map_err(|e| e.to_string())
}

/// A SHA-256 written as 64 hex digits (what `og world-server cert` prints).
pub fn parse_hash(hex: &str) -> Result<[u8; 32], String> {
    let hex = hex.trim();
    if hex.len() != 64 {
        return Err(format!("'{hex}' is not a SHA-256 of 64 hex digits"));
    }
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16)
            .map_err(|_| format!("'{hex}' is not hex"))?;
    }
    Ok(out)
}
