//! `<kbd>` in documents (docs/ui.md §2): a keycap, an action's glyph on
//! the device used last, and a rebind capture.
use super::*;

impl Ui {
    /// A `<kbd>` (G7): its text on a keycap, or with `data-action` the
    /// action's glyph on the device used last (its icon where the atlas has
    /// one, else its name on a cap), named by the action's label. With
    /// `data-og-rebind` it is a button: a press captures the next key,
    /// which it writes back as its binding (Escape cancels).
    pub(super) fn kbd(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &str,
        r: &Resolved,
        text: String,
    ) -> crate::Response {
        let action = r.str("dataAction").unwrap_or_default().to_string();
        let glyph = (!action.is_empty()).then(|| self.glyph(&action)).flatten();
        let label = self
            .action_defs()
            .into_iter()
            .find(|d| d.name == action)
            .map(|d| d.label)
            .unwrap_or_default();
        let cap = match &glyph {
            Some(g) if g.text.is_empty() => g.icon.clone(),
            Some(g) => g.text.clone(),
            None if text.is_empty() => action.clone(),
            None => text,
        };
        if r.flag("dataOgRebind") {
            let capturing = format!("{id}~rebind");
            let on = self.memory(&capturing, 0.0) > 0.0;
            if on {
                let input = self.input();
                let escape = input.keys_pressed.contains(&"Escape");
                let key = input.keys_pressed.iter().find(|k| **k != "Escape").copied();
                if escape || key.is_some() {
                    self.remember(&capturing, 0.0);
                }
                if let Some(code) = key {
                    self.write_back(b, n, "binding", serde_json::json!(code));
                }
            }
            let shown = if on { "…".to_string() } else { cap };
            let resp = self.button(
                id,
                &shown,
                crate::ButtonDesc {
                    size: crate::ControlSize::Small,
                    ..crate::ButtonDesc::default()
                },
            );
            if resp.clicked {
                self.remember(&capturing, 1.0);
            }
            let at = self.last_closed;
            self.nodes[at].a11y.name = Some(label);
            return resp;
        }
        match glyph.filter(|g| self.icons().find(&g.icon).is_some()) {
            Some(g) => self.icon(
                id,
                &g.icon,
                crate::widgets::IconDesc {
                    size: 20.0,
                    label: Some(label),
                    ..crate::widgets::IconDesc::default()
                },
            ),
            None => {
                self.badge(id, &cap, crate::widgets::Tone::Neutral);
                let at = self.last_closed;
                if !label.is_empty() {
                    self.nodes[at].a11y.name = Some(label);
                }
            }
        }
        crate::Response::default()
    }
}
