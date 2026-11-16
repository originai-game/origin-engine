//! The native host's HTTP client for the engine's remote files (a
//! document's `url()` image, a package served from an asset base): one GET
//! checked against its Subresource Integrity metadata, as the web engine
//! Worker's `fetch()` checks it.
//!
//! - **Subresource Integrity** (W3C SRI §3.3.5): the bytes must match one
//!   of the strongest hashes the metadata lists (sha256, sha384, sha512);
//!   metadata naming no hash this knows matches anything.
//!
//! The engine sends no cookies, credentials, `Origin` or `Referer`: there
//! is nothing for CORS or a referrer policy to protect natively, so neither
//! is applied (on the web the browser still asks a cross-origin response to
//! allow the page's origin). Redirects are followed. The client is ureq on
//! rustls (ring, with Mozilla's roots from webpki-roots), as the relay
//! client's TLS is.

use std::time::Duration;

/// Redirects followed at most (Fetch §4.4 step 7's 20).
const MAX_REDIRECTS: usize = 20;
/// How long one fetch may take, its body included: a large pack on a
/// slow network fits, only a hung server reaches it (the web boot's
/// `FETCH_DEADLINE_MS`).
const DEADLINE: Duration = Duration::from_secs(300);
/// The largest body read (a streaming pack is the largest file a game has).
const MAX_BYTES: u64 = 2 << 30;

/// One GET.
#[derive(Clone, Debug, Default)]
pub struct Request<'a> {
    pub url: &'a str,
    /// Subresource Integrity metadata the body must match.
    pub integrity: Option<&'a str>,
}

/// Fetches `r.url`'s body, following redirects, and checks its integrity;
/// the error says what refused it.
pub fn get(r: &Request<'_>) -> Result<Vec<u8>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .max_redirects(0)
        .http_status_as_error(false)
        .timeout_global(Some(DEADLINE))
        .user_agent("origin-engine")
        .build()
        .into();
    let mut url = r.url.to_owned();
    for _ in 0..=MAX_REDIRECTS {
        if !matches!(scheme(&url).as_str(), "http" | "https") {
            return Err(format!("{url} is not an http(s) URL"));
        }
        let mut response = agent.get(&url).call().map_err(|e| format!("{url}: {e}"))?;
        let status = response.status().as_u16();
        if matches!(status, 301 | 302 | 303 | 307 | 308) {
            let location = response
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| format!("{url}: HTTP {status} names no Location"))?;
            url = join(&url, location);
            continue;
        }
        if !(200..300).contains(&status) {
            return Err(format!("{url}: HTTP {status}"));
        }
        let bytes = response
            .body_mut()
            .with_config()
            .limit(MAX_BYTES)
            .read_to_vec()
            .map_err(|e| format!("{url}: {e}"))?;
        if let Some(metadata) = r.integrity {
            integrity(&bytes, metadata).map_err(|e| format!("{url}: {e}"))?;
        }
        return Ok(bytes);
    }
    Err(format!("{}: more than {MAX_REDIRECTS} redirects", r.url))
}

/// Subresource Integrity (SRI §3.3.5): `Ok` when `bytes` match one of the
/// strongest hashes in `metadata`, or when it names none this knows.
pub fn integrity(bytes: &[u8], metadata: &str) -> Result<(), String> {
    use ring::digest::{SHA256, SHA384, SHA512, digest};
    let rank = |alg: &str| match alg {
        "sha256" => Some(1),
        "sha384" => Some(2),
        "sha512" => Some(3),
        _ => None,
    };
    let hashes: Vec<(&str, &str)> = metadata
        .split_ascii_whitespace()
        .filter_map(|item| {
            // Options after `?` are ignored (SRI §3.3.3).
            let item = item.split('?').next()?;
            let (alg, value) = item.split_once('-')?;
            rank(alg).map(|_| (alg, value))
        })
        .collect();
    let Some(strongest) = hashes.iter().filter_map(|(a, _)| rank(a)).max() else {
        return Ok(());
    };
    let mut wanted = Vec::new();
    for (alg, value) in hashes.iter().filter(|(a, _)| rank(a) == Some(strongest)) {
        let algorithm = match *alg {
            "sha256" => &SHA256,
            "sha384" => &SHA384,
            _ => &SHA512,
        };
        let actual = og_core::base64::encode(digest(algorithm, bytes).as_ref());
        // base64url too, as browsers accept it.
        let normal: String = value
            .chars()
            .map(|c| match c {
                '-' => '+',
                '_' => '/',
                c => c,
            })
            .collect();
        if actual.trim_end_matches('=') == normal.trim_end_matches('=') {
            return Ok(());
        }
        wanted.push(format!("{alg}-{actual}"));
    }
    Err(format!(
        "refused by integrity: its bytes are {}, not {metadata}",
        wanted.join(" ")
    ))
}

fn scheme(url: &str) -> String {
    url.split_once("://")
        .map(|(s, _)| s.to_ascii_lowercase())
        .unwrap_or_default()
}

/// `location` resolved against `base` (an absolute URL, a path from the
/// root, a scheme-relative URL or a relative path).
pub fn join(base: &str, location: &str) -> String {
    if location.contains("://") {
        return location.to_owned();
    }
    let s = scheme(base);
    let rest = &base[base.find("://").map_or(0, |i| i + 3)..];
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..end];
    if let Some(net) = location.strip_prefix("//") {
        return format!("{s}://{net}");
    }
    if location.starts_with('/') {
        return format!("{s}://{authority}{location}");
    }
    let path = rest[end..].split(['?', '#']).next().unwrap_or("");
    let dir = path.rsplit_once('/').map_or("", |(d, _)| d);
    let mut parts: Vec<&str> = dir.split('/').filter(|p| !p.is_empty()).collect();
    let (rel, query) = location
        .find(['?', '#'])
        .map_or((location, ""), |i| (&location[..i], &location[i..]));
    for c in rel.split('/') {
        match c {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            c => parts.push(c),
        }
    }
    let trailing = if rel.ends_with('/') && !parts.is_empty() {
        "/"
    } else {
        ""
    };
    format!("{s}://{authority}/{}{trailing}{query}", parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrity_takes_the_strongest_hash() {
        // SHA-256 and SHA-384 of "abc" (FIPS 180-2's test vectors).
        let s256 = "sha256-ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0=";
        let s384 = "sha384-ywB1P0WjXou1oD1pmsZQBycsMqsO3tFjGotgWkP/W+2AhgcroefMI1i67KE0yCWn";
        assert_eq!(integrity(b"abc", s256), Ok(()));
        assert_eq!(
            integrity(b"abc", &s256.replace('+', "-").replace('/', "_")),
            Ok(())
        );
        assert!(integrity(b"abd", s256).is_err());
        // Only the strongest algorithm counts: a wrong sha384 fails though
        // the sha256 matches.
        let wrong384 = "sha384-AAAAP0WjXou1oD1pmsZQBycsMqsO3tFjGotgWkP/W+2AhgcroefMI1i67KE0yCWn";
        assert!(integrity(b"abc", &format!("{s256} {wrong384}")).is_err());
        assert_eq!(integrity(b"abc", &format!("{wrong384} {s384}?opt")), Ok(()));
        assert_eq!(
            integrity(b"abc", "md5-xyz"),
            Ok(()),
            "no known hash: matches"
        );
    }

    #[test]
    fn redirects_join_against_the_url() {
        let b = "https://cdn.example/game/cooked/a.ogb?v=1";
        assert_eq!(join(b, "/x.png"), "https://cdn.example/x.png");
        assert_eq!(
            join(b, "../art/x.png"),
            "https://cdn.example/game/art/x.png"
        );
        assert_eq!(join(b, "//other.example/y"), "https://other.example/y");
        assert_eq!(join(b, "http://z/q"), "http://z/q");
    }
}
