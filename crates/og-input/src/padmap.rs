//! Gamepad mappings: what a host's raw pad is in the standard layout.
//!
//! A host sends each pad raw ([`RawPad`]): its buttons, axes and hat in the
//! order its driver lists them, with its SDL GUID and the [`PadLayout`]
//! that order follows. og-input alone turns it into the W3C standard layout
//! og.Gamepad holds ([`map`]), through mappings in SDL_GameControllerDB's
//! syntax: the community database (third_party/SDL_GameControllerDB) for
//! desktop drivers, and og's own lines for the browser's non-standard pads
//! and Windows.Gaming.Input. A pad no line names reads its raw order as the
//! standard one, which is right for most pads and close for the rest.
//!
//! The SDL syntax as read here: `guid,name,` then `element:source` pairs.
//! Elements are SDL's (`a` `b` `x` `y` `back` `guide` `start` `leftstick`
//! `rightstick` `leftshoulder` `rightshoulder` `dpup` `dpdown` `dpleft`
//! `dpright` `leftx` `lefty` `rightx` `righty` `lefttrigger`
//! `righttrigger`; the rest name nothing in the standard layout and are
//! skipped), optionally `+` or `-` for half an axis. Sources are `bN`, `aN`
//! with an optional `+` or `-` half and `~` to invert, and `h0.D` (hat 0,
//! direction 1 up, 2 right, 4 down, 8 left). `platform:`, `crc:`, `hint:`
//! and `sdk…` fields are the database's and select or annotate lines.
//! og's own lines use `platform:Web` (the browser's raw order, its hat as
//! two axes) and `platform:XInput`, and a GUID whose product is 0000 names
//! every product of its vendor.

use crate::event::{GAMEPAD_BUTTONS, GamepadState};
use crate::glyph::Device;
use std::collections::HashMap;
use std::sync::OnceLock;

/// The raw buttons a host sends (more are dropped).
pub const RAW_BUTTONS: usize = 24;
/// The raw axes a host sends, hat 0 apart (more are dropped).
pub const RAW_AXES: usize = 8;

/// The order a raw pad's buttons and axes follow: who listed them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PadLayout {
    /// The W3C standard layout already (a browser's `mapping: "standard"`).
    #[default]
    Standard,
    /// A browser's own order for a pad it does not map.
    Web,
    /// Linux evdev, as SDL numbers it (hat 0 apart from the axes).
    Linux,
    /// DirectInput or raw HID on Windows, as SDL numbers it.
    Windows,
    /// IOKit HID on macOS, as SDL numbers it.
    Mac,
    /// Windows.Gaming.Input's Gamepad: West, South, East, North, LB, RB,
    /// View, Menu, the stick presses, the d-pad up, right, down, left;
    /// left x, y, right x, the triggers, right y (y up).
    XInput,
}

impl PadLayout {
    pub const ALL: [PadLayout; 6] = [
        PadLayout::Standard,
        PadLayout::Web,
        PadLayout::Linux,
        PadLayout::Windows,
        PadLayout::Mac,
        PadLayout::XInput,
    ];
    const NAMES: [&'static str; 6] = ["standard", "web", "linux", "windows", "macos", "xinput"];
    /// The database's `platform:` value for each.
    const PLATFORMS: [&'static str; 6] = ["", "Web", "Linux", "Windows", "Mac OS X", "XInput"];

    pub fn name(self) -> &'static str {
        Self::NAMES[self as usize]
    }

    pub fn from_name(n: &str) -> Option<PadLayout> {
        Self::ALL.into_iter().find(|l| l.name() == n)
    }

    pub fn index(self) -> u32 {
        self as u32
    }

    pub fn from_index(i: u32) -> Option<PadLayout> {
        Self::ALL.get(i as usize).copied()
    }

    fn from_platform(p: &str) -> Option<PadLayout> {
        Self::ALL
            .into_iter()
            .find(|l| *l != PadLayout::Standard && Self::PLATFORMS[*l as usize] == p)
    }
}

/// One pad as its host reads it, before any mapping.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RawPad {
    /// Its slot, 0 to 7: the host keeps a pad's slot while it is
    /// connected, so pads never renumber when another goes.
    pub pad: u32,
    pub connected: bool,
    pub layout: PadLayout,
    /// SDL's GUID: bus, CRC, vendor, product and version (little-endian).
    pub guid: [u8; 16],
    /// Each button's value, 0 to 1.
    pub buttons: [f32; RAW_BUTTONS],
    /// Each axis, -1 to 1 (a trigger on an axis rests at -1).
    pub axes: [f32; RAW_AXES],
    /// Hat 0's held directions: 1 up, 2 right, 4 down, 8 left.
    pub hat: u8,
}

impl RawPad {
    /// SDL's GUID of a USB pad from its vendor and product ids.
    pub fn usb_guid(vendor: u16, product: u16) -> [u8; 16] {
        let mut g = [0u8; 16];
        g[0] = 0x03;
        g[4..6].copy_from_slice(&vendor.to_le_bytes());
        g[8..10].copy_from_slice(&product.to_le_bytes());
        g
    }

    /// A pad in the standard layout: `buttons` a mask of GAMEPAD_BUTTONS,
    /// the sticks y down, the analog triggers (what a playtest sends).
    pub fn standard(pad: u32, buttons: u32, sticks: [f32; 4], triggers: [f32; 2]) -> RawPad {
        let mut raw = RawPad {
            pad,
            connected: true,
            ..RawPad::default()
        };
        for (i, b) in raw
            .buttons
            .iter_mut()
            .enumerate()
            .take(GAMEPAD_BUTTONS.len())
        {
            *b = f32::from(u8::from(buttons & (1 << i) != 0));
        }
        raw.buttons[6] = raw.buttons[6].max(triggers[0]);
        raw.buttons[7] = raw.buttons[7].max(triggers[1]);
        raw.axes[..4].copy_from_slice(&sticks);
        raw
    }

    /// Its vendor id (0 when its GUID does not carry one).
    pub fn vendor(&self) -> u16 {
        u16::from_le_bytes([self.guid[4], self.guid[5]])
    }

    /// Every value is finite and the slot is a pad's.
    pub fn is_valid(&self) -> bool {
        (self.pad as usize) < crate::event::MAX_GAMEPADS
            && self.buttons.iter().chain(&self.axes).all(|v| v.is_finite())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Half {
    Full,
    Positive,
    Negative,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Source {
    Button(u8),
    Axis { index: u8, half: Half, invert: bool },
    Hat(u8),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Target {
    /// A standard button (GAMEPAD_BUTTONS index, never the triggers).
    Button(u8),
    /// Left x, y, right x, y, or a half of one.
    Axis(u8, Half),
    /// The left or right trigger: its value and its button.
    Trigger(u8),
}

/// One pad's mapping: which raw input gives each standard one.
#[derive(Clone, Debug, PartialEq)]
pub struct Mapping {
    /// The database's name for the pad.
    pub name: String,
    entries: Vec<(Source, Target)>,
}

fn target(element: &str) -> Option<Option<Target>> {
    let (half, e) = match element.as_bytes().first() {
        Some(b'+') => (Half::Positive, &element[1..]),
        Some(b'-') => (Half::Negative, &element[1..]),
        _ => (Half::Full, element),
    };
    let axis = ["leftx", "lefty", "rightx", "righty"]
        .iter()
        .position(|n| *n == e);
    if let Some(i) = axis {
        return Some(Some(Target::Axis(i as u8, half)));
    }
    if half != Half::Full {
        return None;
    }
    const BUTTONS: [(&str, u8); 15] = [
        ("a", 0),
        ("b", 1),
        ("x", 2),
        ("y", 3),
        ("leftshoulder", 4),
        ("rightshoulder", 5),
        ("back", 8),
        ("start", 9),
        ("leftstick", 10),
        ("rightstick", 11),
        ("dpup", 12),
        ("dpdown", 13),
        ("dpleft", 14),
        ("dpright", 15),
        ("guide", 16),
    ];
    Some(match e {
        "lefttrigger" => Some(Target::Trigger(0)),
        "righttrigger" => Some(Target::Trigger(1)),
        _ => BUTTONS
            .iter()
            .find(|(n, _)| *n == e)
            .map(|(_, i)| Target::Button(*i)),
    })
}

fn source(s: &str) -> Option<Source> {
    let number = |t: &str| t.parse::<u8>().ok();
    if let Some(n) = s.strip_prefix('b') {
        return number(n).map(Source::Button);
    }
    if let Some(h) = s.strip_prefix("h0.") {
        return number(h)
            .filter(|d| [1, 2, 4, 8].contains(d))
            .map(Source::Hat);
    }
    let (half, rest) = match s.as_bytes().first() {
        Some(b'+') => (Half::Positive, &s[1..]),
        Some(b'-') => (Half::Negative, &s[1..]),
        _ => (Half::Full, s),
    };
    let (rest, invert) = match rest.strip_suffix('~') {
        Some(r) => (r, true),
        None => (rest, false),
    };
    let index = number(rest.strip_prefix('a')?)?;
    Some(Source::Axis {
        index,
        half,
        invert,
    })
}

/// One database line: its GUID, its platform (`None` for a phone's, which
/// no og host runs on) and its mapping. SDL's own `xinput` line (a name in
/// place of a GUID, the layout og's `platform:XInput` line gives), a hat
/// past 0 or a source this reads no other way is `Err`.
pub fn parse_line(line: &str) -> Result<([u8; 16], Option<PadLayout>, Mapping), String> {
    let mut fields = line.split(',');
    let guid_text = fields.next().unwrap_or("");
    let mut guid = [0u8; 16];
    if guid_text.len() != 32 {
        return Err(format!("'{guid_text}' is not a 32-digit GUID"));
    }
    for (i, b) in guid.iter_mut().enumerate() {
        *b = u8::from_str_radix(&guid_text[2 * i..2 * i + 2], 16)
            .map_err(|_| format!("'{guid_text}' is not a GUID"))?;
    }
    let name = fields.next().unwrap_or("").to_owned();
    let mut platform = None;
    let mut entries = Vec::new();
    for f in fields.filter(|f| !f.is_empty()) {
        let (k, v) = f
            .split_once(':')
            .ok_or_else(|| format!("'{f}' is not element:source"))?;
        match k {
            // No og host runs on the database's phones: their lines name no layout.
            "platform" if v == "Android" || v == "iOS" => platform = None,
            "platform" => {
                platform =
                    Some(PadLayout::from_platform(v).ok_or_else(|| format!("platform '{v}'"))?);
            }
            "crc" | "hint" => {}
            _ if k.starts_with("sdk") => {}
            _ => {
                let t = target(k).ok_or_else(|| format!("element '{k}'"))?;
                let s = source(v).ok_or_else(|| format!("source '{v}' of {k}"))?;
                if let Some(t) = t {
                    entries.push((s, t));
                }
            }
        }
    }
    Ok((guid, platform, Mapping { name, entries }))
}

/// og's own lines: the browser's non-standard pads (by vendor, any
/// product unless one is named) and Windows.Gaming.Input's Gamepad.
const OG_LINES: &str = "\
030000005e0400000000000000000000,Xbox Controller,a:b0,b:b1,x:b2,y:b3,leftshoulder:b4,rightshoulder:b5,lefttrigger:a2,righttrigger:a5,back:b6,start:b7,guide:b8,leftstick:b9,rightstick:b10,dpup:-a7,dpdown:+a7,dpleft:-a6,dpright:+a6,leftx:a0,lefty:a1,rightx:a3,righty:a4,platform:Web,
030000004c050000c405000000000000,PS4 Controller,a:b0,b:b1,x:b3,y:b2,leftshoulder:b4,rightshoulder:b5,lefttrigger:a2,righttrigger:a5,back:b8,start:b9,guide:b10,leftstick:b11,rightstick:b12,dpup:-a7,dpdown:+a7,dpleft:-a6,dpright:+a6,leftx:a0,lefty:a1,rightx:a3,righty:a4,platform:Web,
030000004c050000cc09000000000000,PS4 Controller,a:b0,b:b1,x:b3,y:b2,leftshoulder:b4,rightshoulder:b5,lefttrigger:a2,righttrigger:a5,back:b8,start:b9,guide:b10,leftstick:b11,rightstick:b12,dpup:-a7,dpdown:+a7,dpleft:-a6,dpright:+a6,leftx:a0,lefty:a1,rightx:a3,righty:a4,platform:Web,
030000004c050000a00b000000000000,PS4 Controller,a:b0,b:b1,x:b3,y:b2,leftshoulder:b4,rightshoulder:b5,lefttrigger:a2,righttrigger:a5,back:b8,start:b9,guide:b10,leftstick:b11,rightstick:b12,dpup:-a7,dpdown:+a7,dpleft:-a6,dpright:+a6,leftx:a0,lefty:a1,rightx:a3,righty:a4,platform:Web,
030000004c050000e60c000000000000,PS5 Controller,a:b0,b:b1,x:b3,y:b2,leftshoulder:b4,rightshoulder:b5,lefttrigger:a2,righttrigger:a5,back:b8,start:b9,guide:b10,leftstick:b11,rightstick:b12,dpup:-a7,dpdown:+a7,dpleft:-a6,dpright:+a6,leftx:a0,lefty:a1,rightx:a3,righty:a4,platform:Web,
030000004c050000f20d000000000000,PS5 Controller,a:b0,b:b1,x:b3,y:b2,leftshoulder:b4,rightshoulder:b5,lefttrigger:a2,righttrigger:a5,back:b8,start:b9,guide:b10,leftstick:b11,rightstick:b12,dpup:-a7,dpdown:+a7,dpleft:-a6,dpright:+a6,leftx:a0,lefty:a1,rightx:a3,righty:a4,platform:Web,
030000007e0500000000000000000000,Nintendo Switch Pro Controller,a:b0,b:b1,x:b2,y:b3,leftshoulder:b4,rightshoulder:b5,lefttrigger:b6,righttrigger:b7,back:b8,start:b9,leftstick:b10,rightstick:b11,dpup:b12,dpdown:b13,dpleft:b14,dpright:b15,guide:b16,leftx:a0,lefty:a1,rightx:a2,righty:a3,platform:Web,
030000006d0400000000000000000000,Logitech Gamepad,a:b0,b:b1,x:b2,y:b3,leftshoulder:b4,rightshoulder:b5,lefttrigger:a2,righttrigger:a5,back:b6,start:b7,guide:b8,leftstick:b9,rightstick:b10,dpup:-a7,dpdown:+a7,dpleft:-a6,dpright:+a6,leftx:a0,lefty:a1,rightx:a3,righty:a4,platform:Web,
030000006f0e00000000000000000000,PDP Xbox Controller,a:b0,b:b1,x:b2,y:b3,leftshoulder:b4,rightshoulder:b5,lefttrigger:a2,righttrigger:a5,back:b6,start:b7,guide:b8,leftstick:b9,rightstick:b10,dpup:-a7,dpdown:+a7,dpleft:-a6,dpright:+a6,leftx:a0,lefty:a1,rightx:a3,righty:a4,platform:Web,
03000000c62400000000000000000000,PowerA Xbox Controller,a:b0,b:b1,x:b2,y:b3,leftshoulder:b4,rightshoulder:b5,lefttrigger:a2,righttrigger:a5,back:b6,start:b7,guide:b8,leftstick:b9,rightstick:b10,dpup:-a7,dpdown:+a7,dpleft:-a6,dpright:+a6,leftx:a0,lefty:a1,rightx:a3,righty:a4,platform:Web,
03000000c82d00000000000000000000,8BitDo Xbox Controller,a:b0,b:b1,x:b2,y:b3,leftshoulder:b4,rightshoulder:b5,lefttrigger:a2,righttrigger:a5,back:b6,start:b7,guide:b8,leftstick:b9,rightstick:b10,dpup:-a7,dpdown:+a7,dpleft:-a6,dpright:+a6,leftx:a0,lefty:a1,rightx:a3,righty:a4,platform:Web,
00000000000000000000000000000000,Xbox Controller,a:b1,b:b2,x:b0,y:b3,leftshoulder:b4,rightshoulder:b5,back:b6,start:b7,leftstick:b8,rightstick:b9,dpup:b10,dpright:b11,dpdown:b12,dpleft:b13,leftx:a0,lefty:a1~,rightx:a2,lefttrigger:a3,righttrigger:a4,righty:a5~,platform:XInput,
";

/// The community database, on the platforms whose drivers it describes.
#[cfg(not(target_arch = "wasm32"))]
const SDL_DB: &str = include_str!("../../../third_party/SDL_GameControllerDB/gamecontrollerdb.txt");
#[cfg(target_arch = "wasm32")]
const SDL_DB: &str = "";

/// A GUID with its CRC and version cleared: how SDL matches a pad whose
/// firmware differs from the line's.
fn loose(g: [u8; 16]) -> [u8; 16] {
    let mut g = g;
    g[2] = 0;
    g[3] = 0;
    g[12] = 0;
    g[13] = 0;
    g
}

type Db = HashMap<(PadLayout, [u8; 16]), Mapping>;

fn db() -> &'static Db {
    static DB: OnceLock<Db> = OnceLock::new();
    DB.get_or_init(|| {
        let mut out = Db::new();
        // og's lines first: they win over the database's.
        for line in OG_LINES.lines().chain(SDL_DB.lines()) {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // A database line this cannot read (a second hat) is skipped:
            // that pad reads in its raw order.
            let Ok((guid, Some(layout), m)) = parse_line(line) else {
                continue;
            };
            out.entry((layout, guid)).or_insert_with(|| m.clone());
            out.entry((layout, loose(guid))).or_insert(m);
        }
        out
    })
}

/// The mapping a raw pad reads through: its exact GUID, then without its
/// CRC and version, then (og's lines) its vendor's; `None` reads its raw
/// order as the standard one.
pub fn mapping(raw: &RawPad) -> Option<&'static Mapping> {
    if raw.layout == PadLayout::Standard {
        return None;
    }
    let db = db();
    let mut vendor = loose(raw.guid);
    vendor[8] = 0;
    vendor[9] = 0;
    if raw.layout == PadLayout::XInput {
        vendor = [0; 16];
    }
    [raw.guid, loose(raw.guid), vendor]
        .iter()
        .find_map(|g| db.get(&(raw.layout, *g)))
}

/// A pad's family: by its vendor (Microsoft, Sony, Nintendo), else by the
/// name its mapping gives, else a generic pad.
pub fn family(raw: &RawPad, mapping: Option<&Mapping>) -> Device {
    match raw.vendor() {
        0x045e => return Device::Xbox,
        0x054c => return Device::PlayStation,
        0x057e => return Device::Switch,
        _ => {}
    }
    if raw.layout == PadLayout::XInput {
        return Device::Xbox;
    }
    let name = mapping.map(|m| m.name.to_lowercase()).unwrap_or_default();
    let has = |words: &[&str]| words.iter().any(|w| name.contains(w));
    if has(&["xbox", "xinput", "x-box"]) {
        Device::Xbox
    } else if has(&["playstation", "dualshock", "dualsense", "ps3", "ps4", "ps5"]) {
        Device::PlayStation
    } else if has(&["switch", "pro controller", "joy-con", "nintendo"]) {
        Device::Switch
    } else {
        Device::Gamepad
    }
}

/// A raw pad in the standard layout og.Gamepad holds.
pub fn map(raw: &RawPad) -> GamepadState {
    let m = mapping(raw);
    let mut out = GamepadState {
        pad: raw.pad,
        connected: raw.connected,
        device: family(raw, m),
        ..GamepadState::default()
    };
    if !raw.connected {
        return out;
    }
    let read = |s: Source| match s {
        Source::Button(i) => raw.buttons.get(i as usize).copied().unwrap_or(0.0),
        Source::Hat(d) => f32::from(u8::from(raw.hat & d != 0)),
        Source::Axis {
            index,
            half,
            invert,
        } => {
            let v = raw.axes.get(index as usize).copied().unwrap_or(0.0);
            let v = if invert { -v } else { v };
            match half {
                Half::Full => v,
                Half::Positive => v.max(0.0),
                Half::Negative => (-v).max(0.0),
            }
        }
    };
    let mut set = |s: Source, t: Target| {
        let v = read(s);
        match t {
            Target::Button(i) => {
                if v > 0.5 {
                    out.buttons |= 1 << i;
                }
            }
            Target::Axis(i, half) => {
                let a = &mut out.axes[i as usize];
                match half {
                    Half::Full => *a = v,
                    Half::Positive => *a += v.abs(),
                    Half::Negative => *a -= v.abs(),
                }
            }
            Target::Trigger(i) => {
                // A whole axis rests at -1: it reads 0 to 1 across its range.
                let v = match s {
                    Source::Axis {
                        half: Half::Full, ..
                    } => (v + 1.0) / 2.0,
                    _ => v,
                };
                let v = v.clamp(0.0, 1.0);
                out.triggers[i as usize] = v;
                if v > 0.5 {
                    out.buttons |= 1 << (6 + i);
                }
            }
        }
    };
    match m {
        Some(m) => {
            for (s, t) in &m.entries {
                set(*s, *t);
            }
        }
        None => {
            for i in 0..GAMEPAD_BUTTONS.len() as u8 {
                let t = match i {
                    6 | 7 => Target::Trigger(i - 6),
                    _ => Target::Button(i),
                };
                set(Source::Button(i), t);
            }
            for i in 0..4u8 {
                let s = Source::Axis {
                    index: i,
                    half: Half::Full,
                    invert: false,
                };
                set(s, Target::Axis(i, Half::Full));
            }
        }
    }
    for a in &mut out.axes {
        *a = a.clamp(-1.0, 1.0);
    }
    out
}
