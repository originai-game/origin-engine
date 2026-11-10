//! CSS colours (og-css's [`og_css::color`]) resolved against og-ui's
//! style: the theme's palette roles, ink and scheme.

pub use og_css::color::*;

use crate::theme::Srgba;

impl ColorTokens for crate::Theme {
    fn color(&self, name: &str) -> Option<Srgba> {
        crate::Theme::color(self, name)
    }
    fn ink(&self) -> Srgba {
        self.palette.ink
    }
    fn dark(&self) -> bool {
        self.scheme == crate::Scheme::Dark
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_and_light_dark_resolve_against_the_style() {
        let light = crate::Theme::new(crate::Scheme::Light);
        let dark = crate::Theme::new(crate::Scheme::Dark);
        let c = parse_str("light-dark(white, var(--ink))").unwrap();
        assert!(c.literal().is_none());
        assert_eq!(c.resolve(&Context::of(&light)).rgb, [255; 3]);
        assert_eq!(c.resolve(&Context::of(&dark)), dark.palette.ink);
        let mix = parse_str("color-mix(in srgb, var(--panel), black)").unwrap();
        let p = dark.palette.panel.rgb;
        assert_eq!(
            mix.resolve(&Context::of(&dark)).rgb,
            p.map(|v| (f32::from(v) / 2.0).round() as u8)
        );
    }
}
