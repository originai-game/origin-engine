//! Interaction audio is the one pack property without a standard CSS equivalent.
use crate::tokens::{Feedback, SET_SOUNDS, SoundEvent};

pub const GRAMMAR: &str = "none | [ <event> [ <url>+ [volume(<number> <number>?)]? [pitch(<number> <number>?)]? | none ] ]#";

#[derive(Clone, Debug)]
pub struct Sound {
    pub event: SoundEvent,
    pub urls: Vec<String>,
    pub volume: [f32; 2],
    pub pitch: [f32; 2],
}

pub fn parse(value: &str) -> Result<Vec<Sound>, String> {
    use cssparser::{ParseError, Parser, Token};
    if value.trim().eq_ignore_ascii_case("none") {
        return Ok(Vec::new());
    }
    Parser::new(value).parse_comma_separated(|p| {
        let event = p.expect_ident()?.to_string();
        let event = SoundEvent::from_name(&event).ok_or_else(|| ParseError::custom(format!("unknown UI event {event}")))?;
        let mut sound = Sound { event, urls: Vec::new(), volume: [1.0; 2], pitch: [1.0; 2] };
        if p.try_parse(|p| p.expect_ident_matching("none")).is_ok() {
            return Ok(sound);
        }
        let (mut volume, mut pitch) = (false, false);
        while !p.is_exhausted() {
            match p.next()?.clone() {
                Token::UnquotedUrl(url) if !volume && !pitch => sound.urls.push(url.to_string()),
                Token::Function(name) if name.eq_ignore_ascii_case("url") && !volume && !pitch => {
                    sound.urls.push(p.parse_nested_block(|p| Ok::<_, ParseError<String>>(p.expect_string()?.to_string()))?);
                }
                Token::Function(name) if (name.eq_ignore_ascii_case("volume") && !volume && !pitch) || (name.eq_ignore_ascii_case("pitch") && !pitch) => {
                    let is_volume = name.eq_ignore_ascii_case("volume");
                    let range = p.parse_nested_block(|p| {
                        let a = p.expect_number()?;
                        let b = if p.is_exhausted() { a } else { p.expect_number()? };
                        if !a.is_finite() || !b.is_finite() || a > b || (is_volume && (a < 0.0 || b > 1.0)) || (!is_volume && a <= 0.0) {
                            return Err(ParseError::custom("volume is an ordered 0..1 range; pitch is an ordered positive range".to_string()));
                        }
                        Ok([a, b])
                    })?;
                    if is_volume { sound.volume = range; volume = true; } else { sound.pitch = range; pitch = true; }
                }
                _ => return Err(ParseError::custom(GRAMMAR.to_owned())),
            }
        }
        if sound.urls.is_empty() || sound.urls.len() > SET_SOUNDS {
            return Err(ParseError::custom(format!("each event takes 1..{SET_SOUNDS} URLs, or none")));
        }
        Ok(sound)
    }).map_err(|e| format!("-og-sound: {:?}", e.kind))
}

impl Sound {
    pub(crate) fn bind(
        &self,
        sounds: &std::collections::BTreeMap<String, u64>,
        haptic: crate::tokens::Haptic,
    ) -> Feedback {
        let mut feedback = Feedback {
            volume: self.volume,
            pitch: self.pitch,
            haptic,
            ..Feedback::SILENT
        };
        for (i, url) in self.urls.iter().enumerate() {
            feedback.sounds[i] = sounds
                .get(url.trim_start_matches('/'))
                .copied()
                .unwrap_or(0);
        }
        feedback.count = self.urls.len() as u8;
        feedback
    }
}
