//! Token verifiers (docs/design.md §2.12): a world server checks the token
//! in a client's `hello` offline, through the one verifier it was started
//! with, and never calls whoever issued it.
//!
//! - `none` (only with `og world-server --dev`): no token; the client's
//!   token, when it sends one, is its player id and name.
//! - `hs256`: a JWT signed with a secret the server and the game's own
//!   token endpoint share.
//! - `jwks`: a JWT signed with EdDSA (Ed25519) or ES256 (P-256) by a key of
//!   a JWKS document (RFC 7517), found by the token's `kid`: any issuer
//!   that publishes its public keys.
//!
//! Every JWT verifier checks `aud` (the game the server is started for),
//! `exp` and `nbf` (30 s of clock skew), and takes `sub` (the player's
//! opaque, game-scoped id) and the optional `name`, `kind` and `world`.

use crate::shard::Claims;
use og_core::base64;
use ring::{hmac, signature};
use serde_json::Value;

/// Clock skew allowed on `exp` and `nbf`, seconds.
pub const SKEW_S: i64 = 30;

/// Why a token was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthError {
    /// Its `kid` names no key held: the JWKS may have rotated.
    UnknownKey(String),
    Invalid(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::UnknownKey(k) => write!(f, "no key `{k}` in the JWKS"),
            AuthError::Invalid(m) => f.write_str(m),
        }
    }
}

fn bad(m: impl Into<String>) -> AuthError {
    AuthError::Invalid(m.into())
}

/// A public key of a JWKS document.
#[derive(Clone, Debug, PartialEq)]
pub struct Jwk {
    pub kid: String,
    pub alg: JwkAlg,
    /// Ed25519: the 32 bytes; P-256: the uncompressed point (0x04, x, y).
    pub key: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JwkAlg {
    EdDsa,
    Es256,
}

/// Read a JWKS document's Ed25519 and P-256 keys; other keys are skipped.
pub fn parse_jwks(text: &str) -> Result<Vec<Jwk>, String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("the JWKS is not JSON: {e}"))?;
    let keys = v["keys"].as_array().ok_or("the JWKS has no `keys` list")?;
    let field = |k: &Value, n: &str| -> Result<Vec<u8>, String> {
        k[n].as_str()
            .and_then(base64::decode)
            .ok_or_else(|| format!("a key's `{n}` is not base64url"))
    };
    let mut out = Vec::new();
    for k in keys {
        let kid = k["kid"].as_str().unwrap_or("").to_string();
        match (k["kty"].as_str(), k["crv"].as_str()) {
            (Some("OKP"), Some("Ed25519")) => {
                let x = field(k, "x")?;
                if x.len() != 32 {
                    return Err("an Ed25519 key is not 32 bytes".into());
                }
                out.push(Jwk {
                    kid,
                    alg: JwkAlg::EdDsa,
                    key: x,
                });
            }
            (Some("EC"), Some("P-256")) => {
                let (x, y) = (field(k, "x")?, field(k, "y")?);
                if x.len() != 32 || y.len() != 32 {
                    return Err("a P-256 key's x or y is not 32 bytes".into());
                }
                let mut key = vec![4];
                key.extend_from_slice(&x);
                key.extend_from_slice(&y);
                out.push(Jwk {
                    kid,
                    alg: JwkAlg::Es256,
                    key,
                });
            }
            _ => {}
        }
    }
    if out.is_empty() {
        return Err("the JWKS holds no Ed25519 or P-256 key".into());
    }
    Ok(out)
}

/// The claims every JWT verifier holds a token to.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rules {
    /// `aud`: the game the server is started for.
    pub audience: String,
    /// `iss`, when the server names one (`--issuer`).
    pub issuer: Option<String>,
    /// The longest `exp - iat` taken, seconds (`--max-lifetime`); a token
    /// without `iat` is refused when this is set.
    pub max_lifetime_s: Option<i64>,
}

impl Rules {
    pub fn audience(audience: &str) -> Rules {
        Rules {
            audience: audience.into(),
            ..Rules::default()
        }
    }
}

/// How a server checks tokens.
pub enum Verifier {
    None,
    Hs256 { key: hmac::Key, rules: Rules },
    Jwks { keys: Vec<Jwk>, rules: Rules },
}

impl Verifier {
    pub fn hs256(secret: &[u8], rules: Rules) -> Verifier {
        Verifier::Hs256 {
            key: hmac::Key::new(hmac::HMAC_SHA256, secret),
            rules,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Verifier::None => "none",
            Verifier::Hs256 { .. } => "hs256",
            Verifier::Jwks { .. } => "jwks",
        }
    }

    /// The claims of `token` at `now` (UTC seconds).
    pub fn verify(&self, token: &str, now: i64) -> Result<Claims, AuthError> {
        let rules = match self {
            Verifier::None => {
                let id = if token.is_empty() { "guest" } else { token };
                if !(1..=64).contains(&id.len())
                    || !id
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                {
                    return Err(bad(
                        "a dev token is a name: 1 to 64 letters, digits, `.`, `_` and `-`",
                    ));
                }
                return Ok(Claims {
                    player: id.into(),
                    name: id.into(),
                    kind: None,
                    world: None,
                });
            }
            Verifier::Hs256 { rules, .. } | Verifier::Jwks { rules, .. } => rules,
        };
        let mut parts = token.split('.');
        let (Some(h), Some(p), Some(s), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(bad("the token is not a JWT"));
        };
        let header: Value = b64_json(h)?;
        let sig = base64::decode(s).ok_or_else(|| bad("the token's signature is not base64url"))?;
        let signed = &token.as_bytes()[..h.len() + 1 + p.len()];
        let alg = header["alg"].as_str().unwrap_or("");
        match self {
            Verifier::Hs256 { key, .. } => {
                if alg != "HS256" {
                    return Err(bad(format!("the token is signed {alg}, not HS256")));
                }
                hmac::verify(key, signed, &sig)
                    .map_err(|_| bad("the token's signature is wrong"))?;
            }
            Verifier::Jwks { keys, .. } => {
                let want = match alg {
                    "EdDSA" => JwkAlg::EdDsa,
                    "ES256" => JwkAlg::Es256,
                    _ => {
                        return Err(bad(format!(
                            "the token is signed {alg}, not EdDSA or ES256"
                        )));
                    }
                };
                let kid = header["kid"].as_str().unwrap_or("");
                let k = keys
                    .iter()
                    .find(|k| {
                        k.alg == want && (k.kid == kid || (kid.is_empty() && keys.len() == 1))
                    })
                    .ok_or_else(|| AuthError::UnknownKey(kid.into()))?;
                let algorithm: &dyn signature::VerificationAlgorithm = match want {
                    JwkAlg::EdDsa => &signature::ED25519,
                    JwkAlg::Es256 => &signature::ECDSA_P256_SHA256_FIXED,
                };
                signature::UnparsedPublicKey::new(algorithm, &k.key)
                    .verify(signed, &sig)
                    .map_err(|_| bad("the token's signature is wrong"))?;
            }
            Verifier::None => unreachable!(),
        }
        let c: Value = b64_json(p)?;
        let audience = &rules.audience;
        let aud_ok = match &c["aud"] {
            Value::String(a) => a == audience,
            Value::Array(l) => l.iter().any(|a| a.as_str() == Some(audience)),
            _ => false,
        };
        if !aud_ok {
            return Err(bad(format!("the token is not for game `{audience}`")));
        }
        let exp = c["exp"]
            .as_i64()
            .ok_or_else(|| bad("the token has no `exp`"))?;
        if now > exp + SKEW_S {
            return Err(bad("the token has expired"));
        }
        if c["nbf"].as_i64().is_some_and(|nbf| now + SKEW_S < nbf) {
            return Err(bad("the token is not valid yet"));
        }
        if let Some(iss) = &rules.issuer
            && c["iss"].as_str() != Some(iss.as_str())
        {
            return Err(bad(format!("the token is not issued by `{iss}`")));
        }
        if let Some(max) = rules.max_lifetime_s {
            let iat = c["iat"]
                .as_i64()
                .ok_or_else(|| bad("the token has no `iat`"))?;
            if exp - iat > max {
                return Err(bad(format!(
                    "the token lives {} s, over {max} s",
                    exp - iat
                )));
            }
            if iat > now + SKEW_S {
                return Err(bad("the token is issued in the future"));
            }
        }
        let player = c["sub"]
            .as_str()
            .filter(|s| (1..=128).contains(&s.len()))
            .ok_or_else(|| bad("the token has no `sub`"))?;
        Ok(Claims {
            player: player.into(),
            name: c["name"]
                .as_str()
                .map(|n| n.chars().take(64).collect())
                .unwrap_or_default(),
            kind: c["kind"].as_u64().and_then(|k| u32::try_from(k).ok()),
            world: c["world"].as_str().map(str::to_string),
        })
    }
}

fn b64_json(part: &str) -> Result<Value, AuthError> {
    let b = base64::decode(part).ok_or_else(|| bad("a token part is not base64url"))?;
    serde_json::from_slice(&b).map_err(|_| bad("a token part is not JSON"))
}

/// Base64url without padding (RFC 7515 §2), as JWTs are written.
pub fn b64url(b: &[u8]) -> String {
    base64::encode(b)
        .trim_end_matches('=')
        .replace('+', "-")
        .replace('/', "_")
}

/// An HS256 JWT of `claims`: what a self-hoster's token endpoint signs
/// (`og world-server token`, the demo's endpoint, the load test's bots).
pub fn sign_hs256(secret: &[u8], claims: &Value) -> String {
    let head = b64url(br#"{"alg":"HS256","typ":"JWT"}"#);
    let body = b64url(claims.to_string().as_bytes());
    let signed = format!("{head}.{body}");
    let tag = hmac::sign(
        &hmac::Key::new(hmac::HMAC_SHA256, secret),
        signed.as_bytes(),
    );
    format!("{signed}.{}", b64url(tag.as_ref()))
}
