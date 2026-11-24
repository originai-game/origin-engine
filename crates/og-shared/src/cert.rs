//! `og world-server cert`: a self-signed certificate for WebTransport that
//! a browser takes by its SHA-256 hash (`serverCertificateHashes`), with no
//! certificate authority: an ECDSA P-256 key, valid at most 14 days, as the
//! WebTransport spec requires of a certificate named by hash. A server on a
//! public name uses a CA's certificate instead and needs no hash.

/// A certificate, its key (both PEM) and the certificate's SHA-256.
pub struct SelfSigned {
    pub cert_pem: String,
    pub key_pem: String,
    pub sha256: [u8; 32],
}

/// The civil date of a day count since 1970-01-01 (Howard Hinnant's `civil_from_days`).
fn civil(days: i64) -> (i32, u8, u8) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
    let y = (yoe + era * 400 + i64::from(m <= 2)) as i32;
    (y, m, d)
}

/// A certificate for `names` (DNS names or IP addresses), valid from
/// yesterday for `days` days in all (at most 14).
pub fn self_signed(names: &[String], days: u32) -> Result<SelfSigned, String> {
    if !(1..=14).contains(&days) {
        return Err(format!(
            "{days} days: a certificate named by hash lasts 1 to 14 days"
        ));
    }
    let today = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64
        / 86_400;
    let mut params = rcgen::CertificateParams::new(names.to_vec()).map_err(|e| e.to_string())?;
    let (y, m, d) = civil(today - 1);
    params.not_before = rcgen::date_time_ymd(y, m, d);
    let (y, m, d) = civil(today - 1 + i64::from(days) - 1);
    params.not_after = rcgen::date_time_ymd(y, m, d);
    let key =
        rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256).map_err(|e| e.to_string())?;
    let cert = params.self_signed(&key).map_err(|e| e.to_string())?;
    let digest = ring::digest::digest(&ring::digest::SHA256, cert.der());
    let mut sha256 = [0u8; 32];
    sha256.copy_from_slice(digest.as_ref());
    Ok(SelfSigned {
        cert_pem: cert.pem(),
        key_pem: key.serialize_pem(),
        sha256,
    })
}
