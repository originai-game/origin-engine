//! Last-resort vector icons. No atlas, host asset or renderer registration:
//! only a requested fallback parses its SVG, once, on either host.

struct Icon {
    name: &'static str,
    svg: &'static str,
}

macro_rules! icons {
    ($($name:literal),+ $(,)?) => {
        /// The engine's last-resort icon names, after the game's icon layers.
        pub const DEFAULT_ICON_NAMES: &[&str] = &[$($name),+];
        static ICONS: [Icon; DEFAULT_ICON_NAMES.len()] = [$(Icon {
            name: $name,
            svg: include_str!(concat!("../../../third_party/lucide/icons/", $name, ".svg")),
        }),+];
    };
}

icons!(
    "apple",
    "back",
    "bag",
    "bolt",
    "bomb",
    "check",
    "chevronDown",
    "chevronLeft",
    "chevronRight",
    "clock",
    "close",
    "coin",
    "crown",
    "drop",
    "eye",
    "eyeOff",
    "fire",
    "flag",
    "gear",
    "gem",
    "heart",
    "home",
    "key",
    "leaf",
    "lock",
    "map",
    "music",
    "pause",
    "person",
    "play",
    "plus",
    "potion",
    "shield",
    "skull",
    "star",
    "sword",
    "tree",
    "trophy",
    "wheat",
);

/// Built-in icon `name`'s SVG source, for a host that paints its own.
pub fn svg(name: &str) -> Option<&'static str> {
    ICONS
        .iter()
        .find(|icon| icon.name == name)
        .map(|icon| icon.svg)
}

pub(crate) fn find(name: &str) -> Option<&'static str> {
    ICONS
        .iter()
        .find(|icon| icon.name == name)
        .map(|icon| icon.name)
}
