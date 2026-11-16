//! Text drawn straight over the scene. A text whose ancestors draw no
//! ground that holds it (an image or glass under it, or fills that keep it
//! 4.5:1 over a black and a white scene; an image layer that paints
//! elsewhere in its box, such as a filigree corner, is not under it) and whose own effects do not hold it
//! either takes the tag role's treatment from the style's `:root`:
//! `--tag-halo` round its glyphs, under any effects of its own, and, where
//! the style sets one, a `--tag-fill` plate behind it, padded by
//! `--tag-pad` and rounded by `--tag-radius`. The UI cannot see the scene,
//! so it is read at its extremes: a sky as bright as white, a night as
//! dark as black. A halo is read as [`halo_contrast`] reads it, the model
//! `og style check` reports.

use super::*;
use crate::paint::{ColorRef, Paint, TextShadow};

/// WCAG 2's ratio for body text (1.4.3): the bar a text's ground, or its
/// halo, holds it to over the brightest and darkest scenes.
const HOLDS: f32 = 4.5;

/// How far out from the glyphs' edge a halo is read, CSS px.
const HALO_READ: f32 = 1.0;

/// The tag role's treatment, read from the style's `:root` with its
/// theme ([`Ui::read_theme_tokens`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SceneText {
    pub(crate) halo: Vec<TextShadow>,
    pub(crate) plate: Option<Plate>,
}

/// `--tag-fill` behind the text's line box: padded (CSS px, inline and
/// block) and rounded.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Plate {
    pub(crate) color: Srgba,
    pub(crate) pad: [f32; 2],
    pub(crate) radius: f32,
}

type Rgb = [f32; 3];

/// `top` over an opaque `below`, linear light.
fn over(top: [f32; 4], below: Rgb) -> Rgb {
    [0, 1, 2].map(|k| top[k] * top[3] + below[k] * (1.0 - top[3]))
}

fn srgb(c: Rgb) -> Srgba {
    Srgba::from_linear([c[0], c[1], c[2], 1.0])
}

/// The worst WCAG 2 ratio of `ink` (linear) on its halo `e` over the
/// ground `g` (linear): the ground [`HALO_READ`] CSS px out from the
/// glyphs' edge, in the direction its shadows and outline cover least.
/// Each covers there as the overlay shader draws it (overlay.wgsl
/// `fs_msdf`): a shadow, the glyphs moved by its offset with their edge
/// softened over its blur (at least a pixel),
/// `clamp((offset·n − d) / blur + ½)`, so a blurred halo covers at most
/// half at the edge and an offset one lies on one side only; an outline,
/// solid to its width. Taken at one device pixel per CSS px; the font's
/// distance field bounds a blur and an outline further on small text.
pub fn halo_contrast(e: &og_draw::UiTextEffects, ink: [f32; 3], g: [f32; 3]) -> f32 {
    let d = HALO_READ;
    (0..16)
        .map(|k| {
            let a = k as f32 * std::f32::consts::TAU / 16.0;
            let n = og_core::glam::Vec2::new(a.cos(), a.sin());
            let h = e.shadows.iter().fold(g, |g, s| {
                let cover = ((s.offset.dot(n) - d) / s.blur.max(1.0) + 0.5).clamp(0.0, 1.0);
                over([s.color[0], s.color[1], s.color[2], s.color[3] * cover], g)
            });
            let h = e.outline.as_ref().map_or(h, |o| {
                let width = match o.align {
                    og_draw::UiAlign::Inside => 0.0,
                    og_draw::UiAlign::Center => o.width / 2.0,
                    og_draw::UiAlign::Outside => o.width,
                };
                let cover = if width > 0.0 {
                    (width - d + 0.5).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                over([o.color[0], o.color[1], o.color[2], o.color[3] * cover], h)
            });
            srgb(ink).contrast(srgb(h))
        })
        .fold(f32::INFINITY, f32::min)
}

impl Ui {
    /// What `i` and its ancestors lay under its text over a black and a
    /// white scene (linear), or `None` where one of them draws a ground
    /// its author made for what is on it (an image or a vector that paints
    /// under it, glass).
    fn scene_grounds(&self, i: usize, theme: &Theme) -> Option<[Rgb; 2]> {
        // Their solid fills, the innermost first. An ancestor lays its
        // ground only inside its box: a hotkey placed under its slot is
        // over the scene, not over the slot.
        let r = self.tree.nodes[i].placed.rect;
        let (x, y) = (r.x + r.width / 2.0, r.y + r.height / 2.0);
        let mut fills: Vec<[f32; 4]> = Vec::new();
        let mut a = i;
        while a != NONE {
            let n = &self.tree.nodes[a];
            if !n.placed.rect.contains(x, y) {
                a = n.parent;
                continue;
            }
            let mut own = Vec::new();
            if let Some(d) = self.paint.draw_of(a, &self.tree) {
                if d.effects.iter().any(|e| {
                    matches!(
                        e,
                        crate::paint::Effect::Backdrop { .. }
                            | crate::paint::Effect::BackdropFilter(_)
                    )
                }) {
                    return None;
                }
                // A background layer counts where it paints: an image in a
                // corner of a plate leaves its text on the plate's fill.
                let layers = d.fills.iter().chain(d.boxes.iter().flat_map(|b| {
                    b.backgrounds
                        .iter()
                        .filter(|l| l.covers(theme, n.placed.rect, (x, y)))
                        .map(|l| &l.paint)
                }));
                for p in layers {
                    let mut c = match p {
                        Paint::Solid(c) => c.resolve(theme),
                        // A gradient is read at its most transparent stop:
                        // a strip that fades out leaves the scene there.
                        Paint::Gradient(g) | Paint::GradientImage { gradient: g, .. } => g
                            .stops
                            .iter()
                            .map(|s| s.color.resolve(theme))
                            .min_by(|a, b| a[3].total_cmp(&b[3]))
                            .unwrap_or([0.0; 4]),
                        _ => return None,
                    };
                    c[3] *= d.opacity;
                    own.push(c);
                }
            }
            if own.is_empty()
                && let Some(bg) = n.style.sampled.background
            {
                own.push(bg.linear());
            }
            // Drawn bottom first; the list is innermost first.
            fills.extend(own.into_iter().rev());
            a = n.parent;
        }
        Some([[0.0; 3], [1.0; 3]].map(|scene| fills.iter().rev().fold(scene, |g, &c| over(c, g))))
    }

    /// Dresses `i`'s text commands in `cmds` that do not read over the
    /// scene in the style's scene treatment. `grid`: its font's pixel grid
    /// (0: an outline font), whose cells a halo's offsets snap to.
    pub(crate) fn dress_scene_text(
        &self,
        i: usize,
        theme: &Theme,
        cmds: &mut Vec<DrawCmd>,
        grid: u32,
    ) {
        let scene = &self.style.scene_text;
        if scene.halo.is_empty() && scene.plate.is_none() {
            return;
        }
        let mut grounds = None;
        let mut k = 0;
        while k < cmds.len() {
            let (text, own) = match &cmds[k] {
                DrawCmd::Text { .. } => (&cmds[k], None),
                // A gradient ink is the author's: it is left as it is.
                DrawCmd::StyledText { text, effects } if effects.fill.is_none() => {
                    (&**text, Some(&**effects))
                }
                _ => {
                    k += 1;
                    continue;
                }
            };
            let DrawCmd::Text {
                color,
                bounds,
                size,
                clip,
                ..
            } = text
            else {
                k += 1;
                continue;
            };
            let (color, bounds, size, clip) = (*color, *bounds, *size, *clip);
            let Some(g) = *grounds.get_or_insert_with(|| self.scene_grounds(i, theme)) else {
                return;
            };
            let none = og_draw::UiTextEffects::default();
            let holds = color[3] <= 0.0
                || g.iter().all(|&g| {
                    let ink = over(color, g);
                    srgb(ink).contrast(srgb(g)) >= HOLDS
                        || halo_contrast(own.unwrap_or(&none), ink, g) >= HOLDS
                });
            if holds {
                k += 1;
                continue;
            }
            let cell = if grid > 0 { size / grid as f32 } else { 0.0 };
            let snap = |v: f32| crate::paint::snap_to_cells(v, cell);
            // `currentColor` in the halo is the text's own ink.
            let context = crate::css_color::Context {
                current: Srgba::from_linear([color[0], color[1], color[2], 1.0]),
                ..crate::css_color::Context::of(theme)
            };
            let halo = scene.halo.iter().map(|s| og_draw::UiTextShadow {
                offset: og_core::glam::Vec2::from(s.offset.map(snap)),
                blur: if cell > 0.0 { 0.0 } else { s.blur },
                color: match &s.color {
                    ColorRef::Css(c) => c.resolve(&context).linear(),
                    c => c.resolve(theme),
                },
            });
            let dressed = match std::mem::replace(&mut cmds[k], DrawCmd::PopClip) {
                DrawCmd::StyledText { text, mut effects } => {
                    // After its own shadows, which keep their look.
                    effects.shadows.extend(halo);
                    DrawCmd::StyledText { text, effects }
                }
                text if scene.halo.is_empty() => text,
                text => DrawCmd::StyledText {
                    text: Box::new(text),
                    effects: Box::new(og_draw::UiTextEffects {
                        shadows: halo.collect(),
                        fill_box: [bounds.x, bounds.y, bounds.width, bounds.height],
                        transform: [1.0, 0.0, 0.0, 1.0],
                        ..Default::default()
                    }),
                },
            };
            cmds[k] = dressed;
            if let Some(p) = scene.plate {
                cmds.insert(
                    k,
                    DrawCmd::Quad {
                        rect: Rect::new(
                            bounds.x - p.pad[0],
                            bounds.y - p.pad[1],
                            bounds.width + 2.0 * p.pad[0],
                            bounds.height + 2.0 * p.pad[1],
                        ),
                        radius: p.radius,
                        color: p.color.linear(),
                        clip,
                    },
                );
                k += 1;
            }
            k += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn effects(shadows: &[([f32; 2], f32)]) -> og_draw::UiTextEffects {
        og_draw::UiTextEffects {
            shadows: shadows
                .iter()
                .map(|&(o, blur)| og_draw::UiTextShadow {
                    offset: og_core::glam::Vec2::from(o),
                    blur,
                    color: [0.0, 0.0, 0.0, 1.0],
                })
                .collect(),
            ..Default::default()
        }
    }

    /// A halo is read a pixel out from the glyphs' edge, on its weakest
    /// side, as the overlay shader covers it: a blurred shadow covers at
    /// most half at the edge and nothing past its blur, and a drop lies on
    /// one side, so white ink with either over white still reads 1:1;
    /// eight hard copies round the glyphs cover the ring nearly whole (at
    /// least 0.97 black: 1.05 / 0.08 = 13:1 at worst). Counting each
    /// shadow at its full colour read the first two 21:1.
    #[test]
    fn halos_are_read_where_the_glyphs_edge_sits() {
        let (ink, sky) = ([1.0; 3], [1.0; 3]);
        let soft = effects(&[([0.0, 0.0], 2.0), ([0.0, 1.0], 2.0)]);
        let drop = effects(&[([0.0, 2.0], 0.0)]);
        let ring = effects(&[
            ([1.5, 0.0], 0.0),
            ([-1.5, 0.0], 0.0),
            ([0.0, 1.5], 0.0),
            ([0.0, -1.5], 0.0),
            ([1.0, 1.0], 0.0),
            ([-1.0, 1.0], 0.0),
            ([1.0, -1.0], 0.0),
            ([-1.0, -1.0], 0.0),
        ]);
        assert!(halo_contrast(&soft, ink, sky) < 1.05);
        assert!(halo_contrast(&drop, ink, sky) < 1.05);
        assert!(halo_contrast(&ring, ink, sky) > 12.0);
    }
}
