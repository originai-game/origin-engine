//! A world server's configuration: the worlds it serves, the interest grid,
//! the rates and caps, and the movement rules each avatar kind is held to.
//! Read from a `world-server` `.ogt` file (`og world-server --config F`):
//!
//! ```text
//! type world-server
//! version 1
//! ---
//! worlds "main" "plaza"     # the worlds a client may join (empty: any name)
//! cellSize 128              # the interest grid's cell, metres
//! radius 2                  # cells around a client's own that it sees
//! maxVisible 64             # the most avatars one client is sent
//! tickHz 20                 # the server's snapshot rate
//! bytesPerSecond 32768      # the most bytes a second one client is sent
//! maxPlayers 2000           # the most clients at once
//! teleport any              # any | none: whether a jump past the speed rule is taken
//! bounds -1e7 -1e4 -1e7 1e7 1e4 1e7   # min x y z, max x y z
//! kind 0 {                  # the rules of avatar kind 0
//!   maxSpeed 60             # m/s
//! }
//! ```

use og_core::ogt;

/// The movement rules of one avatar kind.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KindRules {
    /// The fastest the avatar moves, m/s, between two accepted states.
    pub max_speed: f32,
}

impl Default for KindRules {
    fn default() -> KindRules {
        KindRules { max_speed: 60.0 }
    }
}

/// Whether a jump past the speed rule is taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeleportPolicy {
    Any,
    None,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    /// The worlds clients may join; empty takes any name.
    pub worlds: Vec<String>,
    pub cell_size: f64,
    pub radius: i32,
    pub max_visible: usize,
    pub tick_hz: u32,
    pub bytes_per_second: usize,
    pub max_players: usize,
    pub teleport: TeleportPolicy,
    pub bounds_min: [f64; 3],
    pub bounds_max: [f64; 3],
    /// Rules by kind; a kind not named takes kind 0's (or the default).
    pub kinds: Vec<(u32, KindRules)>,
    /// How long a lost client's avatar waits for it to come back, in ticks.
    pub linger_ticks: u32,
    /// The most events a second one client sends.
    pub events_per_second: u32,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            worlds: Vec::new(),
            cell_size: 128.0,
            radius: 2,
            max_visible: 64,
            tick_hz: 20,
            bytes_per_second: 32 * 1024,
            max_players: 2000,
            teleport: TeleportPolicy::Any,
            bounds_min: [-1.0e7, -1.0e4, -1.0e7],
            bounds_max: [1.0e7, 1.0e4, 1.0e7],
            kinds: Vec::new(),
            linger_ticks: 200,
            events_per_second: 10,
        }
    }
}

impl Config {
    pub fn rules(&self, kind: u32) -> KindRules {
        let find = |k| self.kinds.iter().find(|(n, _)| *n == k).map(|(_, r)| *r);
        find(kind).or_else(|| find(0)).unwrap_or_default()
    }

    pub fn serves(&self, world: &str) -> bool {
        self.worlds.is_empty() || self.worlds.iter().any(|w| w == world)
    }

    /// Read a `world-server` document.
    pub fn parse(text: &str) -> Result<Config, String> {
        let doc = ogt::parse(text).map_err(|e| e.to_string())?;
        if doc.ty != "world-server" {
            return Err(format!("a `{}` file, not a `world-server` one", doc.ty));
        }
        let mut c = Config::default();
        let num = |e: &ogt::Entry, i| e.number(i).map(|n| n.as_f64()).map_err(|e| e.to_string());
        for e in &doc.body.entries {
            let at = |m: &str| format!("line {}: {} {m}", e.line, e.key);
            match e.key.as_str() {
                "worlds" => {
                    c.worlds = (0..e.values.len())
                        .map(|i| e.string(i).map(str::to_string).map_err(|x| x.to_string()))
                        .collect::<Result<_, _>>()?
                }
                "cellSize" => c.cell_size = positive(num(e, 0)?, &at)?,
                "radius" => c.radius = whole(num(e, 0)?, 0.0, 8.0, &at)? as i32,
                "maxVisible" => c.max_visible = whole(num(e, 0)?, 1.0, 1024.0, &at)? as usize,
                "tickHz" => c.tick_hz = whole(num(e, 0)?, 1.0, 60.0, &at)? as u32,
                "bytesPerSecond" => {
                    c.bytes_per_second = whole(num(e, 0)?, 1024.0, 1.0e8, &at)? as usize
                }
                "maxPlayers" => c.max_players = whole(num(e, 0)?, 1.0, 1.0e6, &at)? as usize,
                "lingerTicks" => c.linger_ticks = whole(num(e, 0)?, 0.0, 1.0e5, &at)? as u32,
                "eventsPerSecond" => {
                    c.events_per_second = whole(num(e, 0)?, 0.0, 1000.0, &at)? as u32
                }
                "teleport" => {
                    c.teleport = match e.word(0).map_err(|x| x.to_string())? {
                        "any" => TeleportPolicy::Any,
                        "none" => TeleportPolicy::None,
                        _ => return Err(at("takes `any` or `none`")),
                    }
                }
                "bounds" => {
                    let v = (0..6).map(|i| num(e, i)).collect::<Result<Vec<_>, _>>()?;
                    c.bounds_min = [v[0], v[1], v[2]];
                    c.bounds_max = [v[3], v[4], v[5]];
                    if (0..3).any(|a| c.bounds_min[a] >= c.bounds_max[a]) {
                        return Err(at("takes a min below its max on every axis"));
                    }
                }
                "kind" => {
                    let k = whole(num(e, 0)?, 0.0, f64::from(u32::MAX), &at)? as u32;
                    let mut r = KindRules::default();
                    for f in e.block.iter().flat_map(|b| &b.entries) {
                        match f.key.as_str() {
                            "maxSpeed" => r.max_speed = positive(num(f, 0)?, &at)? as f32,
                            other => return Err(at(&format!("has no field `{other}`"))),
                        }
                    }
                    c.kinds.retain(|(n, _)| *n != k);
                    c.kinds.push((k, r));
                }
                _ => return Err(format!("line {}: no setting `{}`", e.line, e.key)),
            }
        }
        Ok(c)
    }
}

fn positive(v: f64, at: &dyn Fn(&str) -> String) -> Result<f64, String> {
    if v.is_finite() && v > 0.0 {
        Ok(v)
    } else {
        Err(at("takes a number above 0"))
    }
}

fn whole(v: f64, lo: f64, hi: f64, at: &dyn Fn(&str) -> String) -> Result<f64, String> {
    if v.fract() == 0.0 && (lo..=hi).contains(&v) {
        Ok(v)
    } else {
        Err(at(&format!("takes a whole number from {lo} to {hi}")))
    }
}
