//! The platform services of G3-6 (docs/platform-asks.md #19): purchases and
//! entitlements, ads, achievements, the player, friends and invites, and
//! share. Each is a call on og.platform's channel (a method name and its
//! arguments; the host attaches the player's game-scoped credential and
//! answers with ids scoped to this game), validated here so a malformed
//! call is refused at once and every host sees the same arguments.
//!
//! The engine keeps no account and no money: an entitlement is always the
//! host's answer. What it keeps is og.PlatformGrants, the purchases this
//! world granted ([`og_platform_grant`]), so a consumable is granted once
//! however its answer or notice comes back (a replay, a relaunch, a save
//! restored from before consume).

use super::{MAX_VALUE, call, request, valid_key, with};
use crate::engine::{engine, running};
use crate::{ok, parse, refuse, wire};
use og_core::Reflect;
use serde_json::{Value, json};

/// og.PlatformGrants' name.
pub const GRANTS: &str = "og.PlatformGrants";
/// How many ids og.PlatformGrants keeps: two u32 each in its 512.
const MAX_GRANTS: usize = 256;

/// `og.PlatformGrants`: the hashes of the granted purchase ids.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.PlatformGrants")]
pub struct PlatformGrants {
    pub ids: og_core::List<u32, 512>,
}

/// Registers og.PlatformGrants at its default (lazy: a world that never
/// grants keeps its type table and hash).
pub(crate) fn enable_grants(world: &mut og_ecs::World) -> og_core::Result<()> {
    world.register::<PlatformGrants>()?;
    if !world.has_resource::<PlatformGrants>() {
        world.insert_resource(PlatformGrants::default())?;
    }
    Ok(())
}

/// An opaque id the host hands back (a purchase, a player): 1 to 128 of
/// letters, digits, `.`, `_`, `-` and `:`.
fn valid_token(id: &str) -> bool {
    (1..=128).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b':'))
}

fn named(what: &str, id: &str) -> Result<(), String> {
    if valid_key(id) {
        Ok(())
    } else {
        Err(format!(
            "'{id}' is not {what}: 1 to 64 of letters, digits, '.', '_' and '-'"
        ))
    }
}

fn token(what: &str, id: &str) -> Result<(), String> {
    if valid_token(id) {
        Ok(())
    } else {
        Err(format!(
            "'{id}' is not {what}: 1 to 128 of letters, digits, '.', '_', '-' and ':', as the host gave it"
        ))
    }
}

/// A request from validated arguments, or the refusal.
fn ask(world: u32, method: &str, args: Result<Value, String>) -> String {
    match args {
        Ok(a) => request(world, method, a),
        Err(e) => refuse(e),
    }
}

// ── purchases ────────────────────────────────────────────────────────────

pub fn og_platform_products(world: u32, desc: Option<String>) -> String {
    ask(
        world,
        "purchase.products",
        (|| {
            let d: wire::ProductsDesc = wire::arg(desc.as_deref(), "desc")?;
            let ids = d.ids;
            if ids.len() > 100 {
                return Err(format!(
                    "{} product ids; products describes at most 100 at once",
                    ids.len()
                ));
            }
            for id in &ids {
                named("a product id", id)?;
            }
            Ok(json!({ "ids": ids }))
        })(),
    )
}

pub fn og_platform_purchase(world: u32, desc: Option<String>) -> String {
    ask(
        world,
        "purchase.buy",
        (|| {
            let d: wire::PurchaseDesc = wire::arg(desc.as_deref(), "desc")?;
            let product = d.product.as_str();
            named("a product id", product)?;
            let quantity = d.quantity.clamp(1, 99);
            Ok(json!({ "product": product, "quantity": quantity }))
        })(),
    )
}

pub fn og_platform_entitlements(world: u32) -> u32 {
    call(world, "purchase.entitlements", json!({}), false).unwrap_or(0)
}

pub fn og_platform_consume(world: u32, purchase: String) -> String {
    ask(
        world,
        "purchase.consume",
        token("a purchase id", &purchase).map(|()| json!({ "purchase": purchase })),
    )
}

/// The two u32 (low, high) of a purchase id's stable 64-bit hash.
fn grant_key(purchase: &str) -> [u32; 2] {
    let h = og_core::hash::hash_bytes(purchase.as_bytes());
    [h as u32, (h >> 32) as u32]
}

fn grant_ids(v: &Value) -> Vec<u32> {
    v["ids"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_u64().map(|n| n as u32))
                .collect()
        })
        .unwrap_or_default()
}

fn has_grant(ids: &[u32], key: [u32; 2]) -> bool {
    ids.as_chunks::<2>().0.contains(&key)
}

/// og.PlatformGrants as the world (or the running system) holds it,
/// registered first outside a system (inside one, the system's declaration
/// registered it).
fn read_grants(world: u32) -> Result<Vec<u32>, String> {
    let eng = engine(world)?;
    if running(world).is_none() {
        crate::lazy::ensure(&eng, eng.world()?, GRANTS)?;
    }
    crate::api::resource_value(&eng, GRANTS).map(|v| grant_ids(&v))
}

pub fn og_platform_grant(world: u32, purchase: String) -> String {
    let granted = (|| -> Result<bool, String> {
        token("a purchase id", &purchase)?;
        let mut ids = read_grants(world)?;
        let key = grant_key(&purchase);
        if has_grant(&ids, key) {
            return Ok(false);
        }
        ids.extend_from_slice(&key);
        let keep = MAX_GRANTS * 2;
        if ids.len() > keep {
            ids.drain(..ids.len() - keep);
        }
        crate::engine::write_resource_value(world, GRANTS, &json!({ "ids": ids }))?;
        Ok(true)
    })();
    match granted {
        Ok(new) => ok(json!(new)),
        Err(e) => refuse(e),
    }
}

pub fn og_platform_granted(world: u32, purchase: String) -> bool {
    valid_token(&purchase)
        && read_grants(world).is_ok_and(|ids| has_grant(&ids, grant_key(&purchase)))
}

// ── ads ──────────────────────────────────────────────────────────────────

fn ad_args(desc: Option<String>) -> Result<Value, String> {
    let d: wire::AdDesc = wire::arg(desc.as_deref(), "desc")?;
    let kind = d.kind.as_str();
    let placement = d.placement.as_str();
    let placement = if placement.is_empty() {
        "default"
    } else {
        placement
    };
    named("a placement", placement)?;
    Ok(json!({ "kind": kind, "placement": placement }))
}

pub fn og_platform_ad_ready(world: u32, desc: Option<String>) -> String {
    ask(world, "ad.ready", ad_args(desc))
}

pub fn og_platform_show_ad(world: u32, desc: Option<String>) -> String {
    ask(world, "ad.show", ad_args(desc))
}

// ── achievements ─────────────────────────────────────────────────────────

pub fn og_platform_achievement_unlock(world: u32, id: String) -> String {
    ask(
        world,
        "achievement.unlock",
        named("an achievement id", &id).map(|()| json!({ "id": id })),
    )
}

pub fn og_platform_achievement_progress(world: u32, desc: Option<String>) -> String {
    ask(
        world,
        "achievement.progress",
        (|| {
            let raw = parse(desc.as_deref())?;
            let d: wire::AchievementProgressDesc = wire::value(&raw, "desc")?;
            let id = d.id.as_str();
            named("an achievement id", id)?;
            // Read as f64, as the host is sent it: the IDL's percent is an f32.
            let percent = raw["percent"].as_f64().unwrap_or(0.0);
            if !(0.0..=100.0).contains(&percent) {
                return Err(format!("percent is 0 to 100, got {percent}"));
            }
            Ok(json!({ "id": id, "percent": percent }))
        })(),
    )
}

pub fn og_platform_achievements(world: u32) -> u32 {
    call(world, "achievement.list", json!({}), false).unwrap_or(0)
}

// ── the player, friends and invites ──────────────────────────────────────

pub fn og_platform_player(world: u32) -> u32 {
    call(world, "player.get", json!({}), false).unwrap_or(0)
}

pub fn og_platform_friends(world: u32, desc: Option<String>) -> u32 {
    let d: wire::FriendsDesc = wire::arg(desc.as_deref(), "desc").unwrap_or_default();
    let count = d.count.clamp(1, 200);
    call(world, "friends.list", json!({ "count": count }), false).unwrap_or(0)
}

/// An invite's payload, encoded, at most this.
const MAX_PAYLOAD: usize = 1024;

pub fn og_platform_invite_send(world: u32, desc: Option<String>) -> String {
    ask(
        world,
        "invite.send",
        (|| {
            let d: wire::InviteDesc = wire::arg(desc.as_deref(), "desc")?;
            let to = d.to.as_str();
            if !to.is_empty() {
                token("a friend's player id", to)?;
            }
            let message = d.message.as_str();
            if message.chars().count() > 120 {
                return Err("an invite's message is at most 120 characters".into());
            }
            let payload = d.payload.clone();
            let size = payload.to_string().len();
            if size > MAX_PAYLOAD {
                return Err(format!(
                    "the payload is {size} bytes encoded; an invite carries at most {MAX_PAYLOAD}"
                ));
            }
            Ok(json!({
                "to": if to.is_empty() { Value::Null } else { json!(to) },
                "message": message,
                "payload": payload,
            }))
        })(),
    )
}

pub fn og_platform_invites(world: u32) -> u32 {
    call(world, "invite.list", json!({}), false).unwrap_or(0)
}

// ── share ────────────────────────────────────────────────────────────────

pub fn og_platform_share(world: u32, desc: Option<String>) -> String {
    ask(
        world,
        "share",
        (|| {
            let d: wire::ShareDesc = wire::arg(desc.as_deref(), "desc")?;
            let text = d.text.trim();
            let chars = text.chars().count();
            if chars == 0 || chars > 280 {
                return Err(format!(
                    "a share's text is 1 to 280 characters, got {chars}"
                ));
            }
            let title = d.title.as_str();
            if title.chars().count() > 80 {
                return Err("a share's title is at most 80 characters".into());
            }
            let path = d.path.as_str();
            let path_ok = path.len() <= 256
                && path.bytes().all(|b| {
                    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b'/' | b'=')
                });
            if !path_ok {
                return Err(format!(
                    "'{path}' is not a share path: at most 256 of letters, digits, '.', '_', '-', '/' and '='"
                ));
            }
            Ok(json!({
                "text": text,
                "title": title,
                "path": path,
                "image": d.image,
            }))
        })(),
    )
}

// ── capabilities ─────────────────────────────────────────────────────────

pub fn og_platform_supports(world: u32, method: String) -> bool {
    with(world, |p| p.capabilities.contains(&method))
}

/// A value too large for a notice's data: the cap a host's notice is held to.
pub(super) const MAX_NOTICE: usize = 4 * MAX_VALUE;
