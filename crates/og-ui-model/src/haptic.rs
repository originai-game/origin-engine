//! The UI's haptic patterns: a pattern's pulses, the named presets and
//! the one each UI event plays (docs/ui.md §3.1).

/// Pulses in a haptic pattern.
pub const PULSES: usize = 4;

/// A named haptic pattern.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HapticPreset {
    /// A light click: slider steps, selection.
    Tick,
    /// A firm click: confirm.
    Tap,
    /// A heavy, low knock: an impact, a big reveal.
    Thud,
    Double,
    /// A sustained buzz.
    Buzz,
    Success,
    Warning,
    Error,
}

/// One pulse of a haptic pattern.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pulse {
    /// From the event, ms.
    pub start_ms: f32,
    pub ms: f32,
    /// The low-frequency (heavy) motor, 0–1.
    pub low: f32,
    /// The high-frequency (light) motor, 0–1; a phone's one motor takes the larger.
    pub high: f32,
}

/// A haptic pattern: up to four pulses.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Haptic {
    pub pulses: [Pulse; PULSES],
    pub count: u8,
}

impl Haptic {
    pub const NONE: Haptic = Haptic {
        pulses: [Pulse {
            start_ms: 0.0,
            ms: 0.0,
            low: 0.0,
            high: 0.0,
        }; PULSES],
        count: 0,
    };

    pub fn pulses(&self) -> &[Pulse] {
        &self.pulses[..usize::from(self.count)]
    }

    fn of(p: &[(f32, f32, f32, f32)]) -> Haptic {
        let mut h = Haptic::NONE;
        for (k, &(start_ms, ms, low, high)) in p.iter().enumerate() {
            h.pulses[k] = Pulse {
                start_ms,
                ms,
                low,
                high,
            };
        }
        h.count = p.len() as u8;
        h
    }

    pub fn preset(p: HapticPreset) -> Haptic {
        match p {
            HapticPreset::Tick => Haptic::of(&[(0.0, 10.0, 0.0, 0.35)]),
            HapticPreset::Tap => Haptic::of(&[(0.0, 18.0, 0.2, 0.6)]),
            HapticPreset::Thud => Haptic::of(&[(0.0, 60.0, 0.9, 0.3)]),
            HapticPreset::Double => Haptic::of(&[(0.0, 18.0, 0.2, 0.6), (70.0, 18.0, 0.2, 0.6)]),
            HapticPreset::Buzz => Haptic::of(&[(0.0, 120.0, 0.5, 0.5)]),
            HapticPreset::Success => Haptic::of(&[(0.0, 20.0, 0.1, 0.5), (90.0, 30.0, 0.3, 0.8)]),
            HapticPreset::Warning => Haptic::of(&[(0.0, 40.0, 0.5, 0.5), (100.0, 40.0, 0.5, 0.5)]),
            HapticPreset::Error => Haptic::of(&[
                (0.0, 60.0, 0.8, 0.4),
                (90.0, 60.0, 0.8, 0.4),
                (180.0, 60.0, 0.8, 0.4),
            ]),
        }
    }
}

/// The pattern UI event `event` plays (og.ui.feedback's names); none for
/// an event without one.
pub fn event_haptic(event: &str) -> Haptic {
    use HapticPreset as H;
    match event {
        "press" | "select" | "cancel" | "back" | "tab" => Some(H::Tick),
        "toggleOn" | "toggleOff" | "sliderTick" | "revealUncommon" => Some(H::Tick),
        "confirm" | "notification" | "revealRare" => Some(H::Tap),
        "warning" => Some(H::Warning),
        "error" => Some(H::Error),
        "success" => Some(H::Success),
        "reward" | "revealEpic" => Some(H::Double),
        "revealLegendary" => Some(H::Thud),
        "revealMythic" => Some(H::Buzz),
        _ => None,
    }
    .map_or(Haptic::NONE, Haptic::preset)
}
