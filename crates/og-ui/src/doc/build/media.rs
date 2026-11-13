//! HTML's `<video>` and `<audio>` (docs/ui.md §2) on the engine's players:
//! `src={$player}` names an entity with og.VideoPlayer (a video's frame
//! draws as the element's image) or og.AudioSource. Svelte's media
//! bindings follow the player: `bind:paused` and `bind:currentTime` play,
//! pause and seek it, `bind:volume` and `bind:muted` set its sound's gain,
//! `bind:duration` and `bind:ended` are written back. Requests reach the
//! player at the next tick as a recorded intent ([`crate::bind::MediaOp`]),
//! so playback stays world state; what the element reads back is the last
//! tick's ([`crate::bind::Env::media`]).

use super::*;
use crate::bind::{MediaOp, MediaState};

/// `m:ss` (or `h:mm:ss`) of whole seconds.
fn clock(s: f64) -> String {
    let s = s.max(0.0).floor() as u64;
    match s / 3600 {
        0 => format!("{}:{:02}", s / 60, s % 60),
        h => format!("{h}:{:02}:{:02}", s / 60 % 60, s % 60),
    }
}

impl Ui {
    /// Builds a `<video>` or `<audio>`; answers its response.
    pub(crate) fn media_element(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &str,
        key: u64,
        r: &Resolved,
        video: bool,
    ) -> crate::ui::Response {
        let player = r.num("mediaSrc").map_or(0.0, f64::from);
        let state = (player != 0.0)
            .then(|| b.env.media(player))
            .flatten()
            .unwrap_or_default();
        // The element's wish: paused until autoplay or a write says play.
        let first = !self.docs.state().media_sent.contains_key(&key);
        let paused = if r.has("paused") {
            r.flag("paused")
        } else {
            !r.flag("autoplay")
        };
        let paused = if first && r.flag("autoplay") && paused {
            self.docs.write_back(b, n, "paused", json!(false));
            false
        } else {
            paused
        };
        let gain = if r.flag("muted") {
            0.0
        } else {
            r.num("volume").unwrap_or(1.0).clamp(0.0, 1.0)
        };
        let repeat = r.flag("loop");
        let mut op = MediaOp {
            player,
            ..MediaOp::default()
        };
        if state.playing == paused && !(state.ended && !paused && !repeat && !first) {
            op.playing = Some(!paused);
        }
        // A currentTime someone else wrote (not the one last read back)
        // that is not the player's is a seek.
        let sent = self.docs.state().media_sent.get(&key).copied();
        let store = r.num("currentTime").map(f64::from);
        let seek = store.filter(|t| {
            sent.is_some_and(|(_, _, read, _)| (t - read).abs() > 1e-3)
                && (t - state.time).abs() > 0.25
        });
        op.seek = seek.map(|t| t.max(0.0));
        if sent.is_none_or(|(g, ..)| g != gain) {
            op.gain = Some(gain);
        }
        if sent.is_none_or(|(_, l, ..)| l != repeat) {
            op.repeat = Some(repeat);
        }
        // While a seek is on its way the player still says its old time.
        let pending = match (seek, sent) {
            (Some(_), _) => 2,
            (None, Some((.., p))) => p.saturating_sub(1),
            (None, None) => 0,
        };
        let read = seek.unwrap_or(if pending > 0 {
            sent.map_or(0.0, |s| s.2)
        } else {
            state.time
        });
        self.docs
            .state_mut()
            .media_sent
            .insert(key, (gain, repeat, read, pending));
        if player != 0.0
            && (op.playing.is_some()
                || op.seek.is_some()
                || op.gain.is_some()
                || op.repeat.is_some())
        {
            self.docs.state_mut().media_ops.push(op);
        }
        self.docs
            .media_write_backs(b, n, r, &state, paused, pending > 0);
        let t = *self.env.theme();
        self.open(
            id,
            Element {
                direction: Direction::Column,
                width: r.size("width").unwrap_or_default(),
                height: r.size("height").unwrap_or_default(),
                gap: t.space.xs,
                ..Element::default()
            },
        );
        let at = self.tree.parent();
        self.tree.semantics_mut(at).role = Some(if video { "img" } else { "group" });
        if let Some(name) = r.str("name") {
            self.tree.semantics_mut(at).name = Some(name.to_string());
        }
        if video {
            // The frame once the player has shown one, else the poster.
            let shown = (state.time > 0.0 || state.playing)
                .then_some(state.texture)
                .flatten();
            let poster = r.handle("poster").and_then(|h| b.env.texture(h));
            self.image(
                "Picture",
                crate::ui::ImageDesc {
                    texture: shown.or(poster).or(state.texture),
                    width: Size::grow(),
                    height: r.size("height").map_or_else(Size::fit, |_| Size::grow()),
                    aspect: r.num("aspect").unwrap_or(16.0 / 9.0),
                    ..Default::default()
                },
            );
        }
        let mut resp = crate::ui::Response::default();
        if r.flag("controls") {
            self.open(
                "Controls",
                Element {
                    direction: Direction::Row,
                    gap: t.space.sm,
                    cross: crate::layout::Cross::Center,
                    ..Element::default()
                },
            );
            let label = if paused { "Play" } else { "Pause" };
            resp = self.button(
                "Play",
                label,
                ButtonDesc {
                    disabled: player == 0.0,
                    ..ButtonDesc::default()
                },
            );
            if resp.clicked {
                self.docs.write_back(b, n, "paused", json!(!paused));
                self.docs.state_mut().media_ops.push(MediaOp {
                    player,
                    playing: Some(paused),
                    ..MediaOp::default()
                });
            }
            let time = if state.duration > 0.0 {
                format!("{} / {}", clock(state.time), clock(state.duration))
            } else {
                clock(state.time)
            };
            self.label("Time", &time, LabelDesc::default());
            self.close();
        }
        self.close();
        resp
    }
}

impl crate::ui::Docs {
    /// Svelte's read-only media bindings, and `paused` and `currentTime`
    /// as the player moves them.
    pub(crate) fn media_write_backs(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        r: &Resolved,
        state: &MediaState,
        paused: bool,
        seeking: bool,
    ) {
        let differs =
            |name: &str, v: f64| r.num(name).is_none_or(|x| (f64::from(x) - v).abs() > 1e-3);
        // A seek the store asked for is read back once the player is there.
        if super::nodes::bound(n, "currentTime") && !seeking && differs("currentTime", state.time) {
            self.write_back(b, n, "currentTime", json!(state.time));
        }
        if super::nodes::bound(n, "duration") && differs("duration", state.duration) {
            self.write_back(b, n, "duration", json!(state.duration));
        }
        if super::nodes::bound(n, "ended") && r.flag("ended") != state.ended {
            self.write_back(b, n, "ended", json!(state.ended));
        }
        // The end pauses a player without loop, as HTML's does.
        if state.ended && !paused && !r.flag("loop") && super::nodes::bound(n, "paused") {
            self.write_back(b, n, "paused", json!(true));
        }
    }
}
