//! A `url()` naming an http(s) resource instead of a file of the game's
//! package, with the request modifiers CSS Values 5 §4.5.4 gives it:
//! `cross-origin()`, `integrity()` and `referrer-policy()`.
//!
//! A document names a remote image by one string, its [`UrlRequest::key`]:
//! the URL alone, or the URL followed by its modifiers in CSS syntax. The
//! host fetches each key once and binds the texture under that key
//! ([`crate::Ui::define_texture`]); until then the layer that names it
//! draws nothing, as an image that has not loaded.

/// The `cross-origin()` modifier: a remote image is always fetched in CORS
/// mode (the engine reads its pixels into a texture, which a browser
/// allows only for a CORS-readable response), so a `url()` without the
/// modifier is fetched as `anonymous`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CrossOrigin {
    /// Credentials only to the page's own origin; a response passes with
    /// `Access-Control-Allow-Origin: *` or the page's origin.
    #[default]
    Anonymous,
    /// Credentials (cookies) to any origin; a response passes only with the
    /// page's origin named and `Access-Control-Allow-Credentials: true`.
    UseCredentials,
}

impl CrossOrigin {
    pub fn name(self) -> &'static str {
        match self {
            CrossOrigin::Anonymous => "anonymous",
            CrossOrigin::UseCredentials => "use-credentials",
        }
    }
}

/// Referrer Policy's values (W3C Referrer Policy §3), which
/// `referrer-policy()` takes.
pub const REFERRER_POLICIES: [&str; 8] = [
    "no-referrer",
    "no-referrer-when-downgrade",
    "same-origin",
    "origin",
    "strict-origin",
    "origin-when-cross-origin",
    "strict-origin-when-cross-origin",
    "unsafe-url",
];

/// One remote `url()`: what to fetch and how.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UrlRequest {
    /// An absolute `http:` or `https:` URL.
    pub url: String,
    pub cross_origin: CrossOrigin,
    /// Subresource Integrity metadata (`sha384-<base64>`, several separated
    /// by spaces); the response's bytes must match one of its strongest.
    pub integrity: Option<String>,
    /// One of [`REFERRER_POLICIES`]; `None` is the default,
    /// `strict-origin-when-cross-origin`.
    pub referrer_policy: Option<String>,
}

/// Whether a `url()` names an http(s) resource rather than a package file.
pub fn is_remote(url: &str) -> bool {
    let lower = |n: usize| url.get(..n).map(str::to_ascii_lowercase);
    lower(7).as_deref() == Some("http://") || lower(8).as_deref() == Some("https://")
}

/// A URL's origin, `scheme://host[:port]` lowercased with a default port
/// dropped (HTML's ASCII serialisation); `None` for a URL that is not
/// http(s) or names no host.
pub fn origin(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    let default = match scheme.as_str() {
        "http" => ":80",
        "https" => ":443",
        _ => return None,
    };
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..end];
    // Userinfo names no origin of its own, and a URL that carries it is
    // refused rather than read.
    if authority.is_empty() || authority.contains('@') || authority.contains(char::is_whitespace) {
        return None;
    }
    let host = authority.to_ascii_lowercase();
    let host = host.strip_suffix(default).unwrap_or(&host);
    Some(format!("{scheme}://{host}"))
}

/// Whether `url` is a secure context's: `https:`, or `http:` to the
/// loopback host (`localhost`, `127.0.0.1`, `[::1]`), as a browser counts
/// one (Secure Contexts §3.2).
pub fn secure(url: &str) -> bool {
    let Some(origin) = origin(url) else {
        return false;
    };
    if origin.starts_with("https://") {
        return true;
    }
    let host = &origin["http://".len()..];
    let host = match host.strip_prefix('[') {
        Some(v6) => v6.split(']').next().unwrap_or(""),
        None => host.split(':').next().unwrap_or(""),
    };
    matches!(host, "localhost" | "127.0.0.1" | "::1")
}

/// `reference` resolved against the absolute http(s) URL `base` (RFC 3986
/// §5.2, without query or fragment merging): an absolute URL is itself, a
/// path from `/` replaces the base's, and a relative path replaces its last
/// segment, `.` and `..` read. Only a remote document's urls are joined
/// (og-ui's `frames` feature).
#[cfg(feature = "frames")]
pub fn join(base: &str, reference: &str) -> String {
    if is_remote(reference) {
        return reference.to_owned();
    }
    let Some(origin) = origin(base) else {
        return reference.to_owned();
    };
    let (path, extra) = match reference.find(['?', '#']) {
        Some(i) => reference.split_at(i),
        None => (reference, ""),
    };
    let mut parts: Vec<&str> = Vec::new();
    if !path.starts_with('/') {
        let rest = &base[base.find("://").map_or(0, |i| i + 3)..];
        let rest = &rest[rest.find('/').unwrap_or(rest.len())..];
        let rest = &rest[..rest.find(['?', '#']).unwrap_or(rest.len())];
        if let Some((dir, _)) = rest.rsplit_once('/') {
            parts.extend(dir.split('/').filter(|p| !p.is_empty()));
        }
    }
    let segments: Vec<&str> = path.split('/').collect();
    for (i, part) in segments.iter().enumerate() {
        match *part {
            "." => {}
            ".." => {
                parts.pop();
            }
            "" if i + 1 < segments.len() => {}
            part => parts.push(part),
        }
    }
    format!("{origin}/{}{extra}", parts.join("/"))
}

impl UrlRequest {
    /// The request for `url` with its modifiers as written (`name`, its one
    /// argument: an identifier, or the string `integrity()` takes).
    /// Modifiers apply to a fetch, so a package file takes none.
    pub fn new(url: &str, modifiers: &[(String, String)]) -> Result<UrlRequest, String> {
        if !modifiers.is_empty() && !is_remote(url) {
            return Err(format!(
                "url({url}): request modifiers apply to an http(s) URL; a file of the package is not fetched"
            ));
        }
        let mut r = UrlRequest {
            url: url.to_owned(),
            ..UrlRequest::default()
        };
        let mut seen = Vec::new();
        for (name, value) in modifiers {
            let name = name.to_ascii_lowercase();
            if seen.contains(&name) {
                return Err(format!("url({url}): {name}() is given twice"));
            }
            match name.as_str() {
                "cross-origin" => {
                    r.cross_origin = match value.to_ascii_lowercase().as_str() {
                        "anonymous" => CrossOrigin::Anonymous,
                        "use-credentials" => CrossOrigin::UseCredentials,
                        v => {
                            return Err(format!(
                                "url({url}): cross-origin() takes anonymous or use-credentials, not '{v}'"
                            ));
                        }
                    }
                }
                "integrity" => {
                    if value.trim().is_empty() {
                        return Err(format!(
                            "url({url}): integrity() takes metadata such as \"sha384-…\""
                        ));
                    }
                    r.integrity = Some(value.trim().to_owned());
                }
                "referrer-policy" => {
                    let v = value.to_ascii_lowercase();
                    if !REFERRER_POLICIES.contains(&v.as_str()) {
                        return Err(format!(
                            "url({url}): referrer-policy() takes one of {}, not '{value}'",
                            REFERRER_POLICIES.join(", ")
                        ));
                    }
                    r.referrer_policy = Some(v);
                }
                other => {
                    return Err(format!(
                        "url({url}): {other}() is not a request modifier (cross-origin(), integrity(), referrer-policy())"
                    ));
                }
            }
            seen.push(name);
        }
        Ok(r)
    }

    /// The one string a document and its host name this request by: the
    /// URL, then each modifier that differs from the default in CSS syntax
    /// (`https://cdn/a.png integrity("sha384-…")`). A URL holds no space,
    /// so the first space ends it.
    pub fn key(&self) -> String {
        let mut key = self.url.clone();
        if self.cross_origin == CrossOrigin::UseCredentials {
            key.push_str(" cross-origin(use-credentials)");
        }
        if let Some(i) = &self.integrity {
            key.push_str(" integrity(\"");
            for c in i.chars() {
                if matches!(c, '"' | '\\') {
                    key.push('\\');
                }
                key.push(c);
            }
            key.push_str("\")");
        }
        if let Some(p) = &self.referrer_policy {
            key.push_str(&format!(" referrer-policy({p})"));
        }
        key
    }

    /// The request a [`UrlRequest::key`] names.
    pub fn from_key(key: &str) -> Result<UrlRequest, String> {
        let (url, rest) = key.split_once(' ').unwrap_or((key, ""));
        let mut modifiers = Vec::new();
        let mut rest = rest.trim_start();
        while !rest.is_empty() {
            let (name, after) = rest
                .split_once('(')
                .ok_or_else(|| format!("{key}: a modifier is name(value)"))?;
            let (value, after) = if let Some(quoted) = after.strip_prefix('"') {
                let mut value = String::new();
                let mut chars = quoted.char_indices();
                let mut end = None;
                while let Some((i, c)) = chars.next() {
                    match c {
                        '\\' => {
                            if let Some((_, e)) = chars.next() {
                                value.push(e);
                            }
                        }
                        '"' => {
                            end = Some(i + 1);
                            break;
                        }
                        c => value.push(c),
                    }
                }
                let end = end.ok_or_else(|| format!("{key}: an unclosed string"))?;
                let after = quoted[end..]
                    .strip_prefix(')')
                    .ok_or_else(|| format!("{key}: integrity(\"…\") closes with )"))?;
                (value, after)
            } else {
                let (value, after) = after
                    .split_once(')')
                    .ok_or_else(|| format!("{key}: {name}( is not closed"))?;
                (value.to_owned(), after)
            };
            modifiers.push((name.trim().to_owned(), value));
            rest = after.trim_start();
        }
        UrlRequest::new(url, &modifiers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_names_its_request_and_reads_back() {
        let r = UrlRequest::new(
            "https://cdn.example/a.png",
            &[
                ("referrer-policy".into(), "No-Referrer".into()),
                ("integrity".into(), "sha256-a\"b sha384-c".into()),
                ("cross-origin".into(), "use-credentials".into()),
            ],
        )
        .unwrap();
        let key = r.key();
        assert_eq!(
            key,
            r#"https://cdn.example/a.png cross-origin(use-credentials) integrity("sha256-a\"b sha384-c") referrer-policy(no-referrer)"#
        );
        assert_eq!(UrlRequest::from_key(&key).unwrap(), r);
        // anonymous is the default and is not written.
        let plain = UrlRequest::new(
            "https://cdn.example/a.png",
            &[("cross-origin".into(), "anonymous".into())],
        )
        .unwrap();
        assert_eq!(plain.key(), "https://cdn.example/a.png");
    }

    #[test]
    fn a_stylesheet_and_a_value_name_one_request_alike() {
        use crate::css::at_rules::{rebase_urls, remote_urls, urls};
        let css = r#"a { background: url("https://cdn.example/a.png" referrer-policy(origin) integrity("sha256-x")), url(https://cdn.example/b.png), url(local.png); }
            b { mask-image: image-set(url("https://cdn.example/c.png" cross-origin(use-credentials)) 2x); }"#;
        let a = r#"https://cdn.example/a.png integrity("sha256-x") referrer-policy(origin)"#;
        let c = "https://cdn.example/c.png cross-origin(use-credentials)";
        assert_eq!(
            remote_urls(css),
            [a, "https://cdn.example/b.png", c],
            "modifiers in any order give one key; a package file is not remote"
        );
        assert_eq!(urls(css), ["local.png"]);
        // The reader rebases a sheet before its values are read: the key
        // survives as the url()'s string, and the value names it again.
        let rebased = rebase_urls("ui/menu.css", css);
        assert!(rebased.contains("/ui/local.png"), "{rebased}");
        assert_eq!(remote_urls(&rebased), [a, "https://cdn.example/b.png", c]);
        let value = |v: &str| match crate::css::paint_css::background(v) {
            Some(Ok((layers, _))) => layers[0].image.urls()[0].to_owned(),
            other => panic!("{v}: {other:?}"),
        };
        let start = rebased.find("url(").unwrap();
        let end = rebased[start..].find("),").unwrap() + start + 1;
        assert_eq!(value(&rebased[start..end]), a);
        assert_eq!(
            value(
                r#"url("https://cdn.example/a.png" integrity("sha256-x") referrer-policy(origin))"#
            ),
            a
        );
        assert!(
            crate::css::paint_css::background(r#"url("coin.png" integrity("sha256-x"))"#)
                .is_some_and(|r| r.is_err()),
            "a package file takes no modifiers"
        );
    }

    #[test]
    fn modifiers_are_refused_where_they_mean_nothing() {
        assert!(UrlRequest::new("ui/coin.png", &[("integrity".into(), "x".into())]).is_err());
        let bad = |n: &str, v: &str| UrlRequest::new("https://a/b", &[(n.into(), v.into())]);
        assert!(bad("cross-origin", "always").is_err());
        assert!(bad("referrer-policy", "never").is_err());
        assert!(bad("priority", "high").is_err());
        assert!(bad("integrity", " ").is_err());
    }
}
