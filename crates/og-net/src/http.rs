//! The native host's HTTP client for the engine's remote files (a
//! document's `url()` image, a package served from an asset base): one GET
//! with the browser's rules, so a native run fetches what the web engine
//! Worker's `fetch()` fetches and refuses what it refuses.
//!
//! - **CORS** (Fetch §4.9): a request to another origin carries `Origin`,
//!   and its response (each redirect's too) passes only when
//!   `Access-Control-Allow-Origin` is `*` or names that origin; with
//!   credentials it must name it and `Access-Control-Allow-Credentials`
//!   must be `true`. A redirect from one other origin to another taints
//!   the origin to `null` (Fetch §4.4 step 12).
//! - **Referrer Policy** (W3C Referrer Policy §8.3): the `Referer` each hop
//!   sends, from the game's own URL, under the policy asked for
//!   (`strict-origin-when-cross-origin` when none).
//! - **Subresource Integrity** (W3C SRI §3.3.5): the bytes must match one
//!   of the strongest hashes the metadata lists (sha256, sha384, sha512);
//!   metadata naming no hash this knows matches anything.
//!
//! There is no cookie jar natively: credentials change only what the CORS
//! check asks of a response. The client is ureq on rustls (ring, with
//! Mozilla's roots from webpki-roots), as the relay client's TLS is.

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
    /// The origin the request is made from (`https://play.example`), or
    /// `None` for an opaque one (a game read from files), sent as `null`.
    pub origin: Option<&'a str>,
    /// The URL the referrer is taken from (the game's own); `None` sends no
    /// `Referer` (a file URL is never sent).
    pub referrer: Option<&'a str>,
    /// `cross-origin(use-credentials)`.
    pub credentials: bool,
    /// Subresource Integrity metadata the body must match.
    pub integrity: Option<&'a str>,
    /// A Referrer Policy value; `None` is `strict-origin-when-cross-origin`.
    pub referrer_policy: Option<&'a str>,
}

/// Fetches `r.url`'s body under CORS, the referrer policy and integrity;
/// the error says which refused it.
pub fn get(r: &Request<'_>) -> Result<Vec<u8>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .max_redirects(0)
        .http_status_as_error(false)
        .timeout_global(Some(DEADLINE))
        .user_agent("origin-engine")
        .build()
        .into();
    let mut url = r.url.to_owned();
    // The origin the request carries: tainted to `null` by a redirect
    // between two other origins.
    let origin = r.origin.map(str::to_owned);
    let mut tainted = false;
    for _ in 0..=MAX_REDIRECTS {
        let target = origin_of(&url).ok_or_else(|| format!("{url} is not an http(s) URL"))?;
        let cors = tainted || origin.as_deref() != Some(target.as_str());
        let mut call = agent.get(&url);
        if cors {
            call = call.header("Origin", origin.as_deref().unwrap_or("null"));
        }
        if let Some(referrer) = r
            .referrer
            .and_then(|from| referrer(r.referrer_policy, from, &url))
        {
            call = call.header("Referer", &referrer);
        }
        let mut response = call.call().map_err(|e| format!("{url}: {e}"))?;
        let status = response.status().as_u16();
        let header = |name: &str| {
            response
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        if cors {
            let sent = if tainted {
                "null".to_owned()
            } else {
                origin.clone().unwrap_or_else(|| "null".into())
            };
            cors_check(
                &url,
                &sent,
                r.credentials,
                header("access-control-allow-origin").as_deref(),
                header("access-control-allow-credentials").as_deref(),
            )?;
        }
        if matches!(status, 301 | 302 | 303 | 307 | 308) {
            let location = header("location")
                .ok_or_else(|| format!("{url}: HTTP {status} names no Location"))?;
            let next = join(&url, &location);
            let next_origin =
                origin_of(&next).ok_or_else(|| format!("{url}: redirected to {next}"))?;
            if cors && userinfo(&next) {
                return Err(format!(
                    "{url}: a cross-origin redirect to a URL with credentials ({next})"
                ));
            }
            if next_origin != target && origin.as_deref() != Some(target.as_str()) {
                tainted = true;
            }
            url = next;
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

/// Fetch §4.9's CORS check of one response to a request from `origin`.
fn cors_check(
    url: &str,
    origin: &str,
    credentials: bool,
    allow_origin: Option<&str>,
    allow_credentials: Option<&str>,
) -> Result<(), String> {
    let refused = |why: String| {
        Err(format!(
            "{url}: refused by CORS: {why} (the server must allow the game's origin, {origin})"
        ))
    };
    let Some(allowed) = allow_origin.map(str::trim) else {
        return refused("the response has no Access-Control-Allow-Origin".into());
    };
    if allowed == "*" && !credentials {
        return Ok(());
    }
    if allowed != origin {
        return refused(if allowed == "*" {
            "cross-origin(use-credentials) needs Access-Control-Allow-Origin to name the origin, not *"
                .into()
        } else {
            format!("Access-Control-Allow-Origin is {allowed}")
        });
    }
    if credentials && allow_credentials.map(str::trim) != Some("true") {
        return refused(
            "cross-origin(use-credentials) needs Access-Control-Allow-Credentials: true".into(),
        );
    }
    Ok(())
}

/// The `Referer` a request from `from` to `to` sends under `policy`
/// (Referrer Policy §8.3), or none. The referrer is `from` without its
/// fragment and user info, or only its origin.
pub fn referrer(policy: Option<&str>, from: &str, to: &str) -> Option<String> {
    let full = strip(from)?;
    let origin = format!("{}/", origin_of(from)?);
    let same = origin_of(from) == origin_of(to);
    let downgrade = scheme(from) == "https" && scheme(to) != "https";
    match policy.unwrap_or("strict-origin-when-cross-origin") {
        "no-referrer" => None,
        "no-referrer-when-downgrade" => (!downgrade).then_some(full),
        "same-origin" => same.then_some(full),
        "origin" => Some(origin),
        "strict-origin" => (!downgrade).then_some(origin),
        "origin-when-cross-origin" => Some(if same { full } else { origin }),
        "unsafe-url" => Some(full),
        // strict-origin-when-cross-origin
        _ if same => Some(full),
        _ => (!downgrade).then_some(origin),
    }
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

/// An http(s) URL's origin as `Origin` and `Access-Control-Allow-Origin`
/// write it: scheme and host lowercased, the port only when it is not the
/// scheme's default. `None` for any other URL.
pub fn origin_of(url: &str) -> Option<String> {
    let scheme = scheme(url);
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let rest = &url[url.find("://")? + 3..];
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host_port = authority.rsplit('@').next().unwrap_or("");
    let default = if scheme == "https" { "443" } else { "80" };
    let (host, port) = match host_port.rsplit_once(':') {
        // An IPv6 literal's colons are inside its brackets.
        Some((h, p)) if !p.contains(']') => (h, Some(p)),
        _ => (host_port, None),
    };
    if host.is_empty() {
        return None;
    }
    let host = host.to_ascii_lowercase();
    Some(match port.filter(|p| !p.is_empty() && *p != default) {
        Some(p) => format!("{scheme}://{host}:{p}"),
        None => format!("{scheme}://{host}"),
    })
}

fn scheme(url: &str) -> String {
    url.split_once("://")
        .map(|(s, _)| s.to_ascii_lowercase())
        .unwrap_or_default()
}

fn userinfo(url: &str) -> bool {
    url.split_once("://")
        .map(|(_, rest)| rest.split(['/', '?', '#']).next().unwrap_or(""))
        .is_some_and(|authority| authority.contains('@'))
}

/// `url` as a referrer: without its fragment and user info.
fn strip(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let rest = rest.split('#').next().unwrap_or("");
    let end = rest.find(['/', '?']).unwrap_or(rest.len());
    let host = rest[..end].rsplit('@').next().unwrap_or("");
    let path = &rest[end..];
    Some(format!(
        "{scheme}://{host}{}",
        if path.is_empty() { "/" } else { path }
    ))
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
    fn referrers_follow_each_policy() {
        let from = "https://play.example/game/index.html#start";
        let same = "https://play.example/art/a.png";
        let other = "https://cdn.example/a.png";
        let down = "http://cdn.example/a.png";
        let full = "https://play.example/game/index.html";
        let origin = "https://play.example/";
        let cases: [(Option<&str>, [Option<&str>; 3]); 8] = [
            (None, [Some(full), Some(origin), None]),
            (Some("no-referrer"), [None, None, None]),
            (
                Some("no-referrer-when-downgrade"),
                [Some(full), Some(full), None],
            ),
            (Some("same-origin"), [Some(full), None, None]),
            (Some("origin"), [Some(origin), Some(origin), Some(origin)]),
            (Some("strict-origin"), [Some(origin), Some(origin), None]),
            (
                Some("origin-when-cross-origin"),
                [Some(full), Some(origin), Some(origin)],
            ),
            (Some("unsafe-url"), [Some(full), Some(full), Some(full)]),
        ];
        for (policy, want) in cases {
            for (to, want) in [same, other, down].into_iter().zip(want) {
                assert_eq!(
                    referrer(policy, from, to).as_deref(),
                    want,
                    "{policy:?} {to}"
                );
            }
        }
    }

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
    fn cors_allows_what_a_browser_allows() {
        let o = "https://play.example";
        let check = |cred, ao: Option<&str>, ac: Option<&str>| {
            cors_check("https://cdn.example/a", o, cred, ao, ac).is_ok()
        };
        assert!(check(false, Some("*"), None));
        assert!(check(false, Some(o), None));
        assert!(!check(false, None, None));
        assert!(!check(false, Some("https://other.example"), None));
        assert!(!check(true, Some("*"), Some("true")));
        assert!(!check(true, Some(o), None));
        assert!(check(true, Some(o), Some("true")));
    }

    #[test]
    fn urls_join_and_name_their_origin() {
        assert_eq!(
            origin_of("HTTPS://CDN.Example:443/a?b").as_deref(),
            Some("https://cdn.example")
        );
        assert_eq!(
            origin_of("http://127.0.0.1:8080/x").as_deref(),
            Some("http://127.0.0.1:8080")
        );
        assert_eq!(origin_of("file:///x"), None);
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
