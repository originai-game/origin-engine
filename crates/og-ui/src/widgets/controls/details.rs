//! The disclosure (HTML `<details>` and `<summary>`): a summary row that
//! a press, Enter, Space or A opens and closes, and content built under it
//! that lays out, draws and takes input only while open.

use super::*;

/// A disclosure: its summary and whether it is open.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DetailsDesc {
    /// The summary row's text; empty is `Details` (HTML's default).
    pub summary: String,
    /// Open now (the caller owns it: write back `toggled`).
    pub open: bool,
    pub disabled: bool,
    pub width: Option<Size>,
}

/// A disclosure's state after this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DetailsResult {
    /// Open after this tick's input.
    pub open: bool,
    /// The summary was activated this tick (HTML's `toggle` event).
    pub toggled: bool,
}

impl Ui {
    /// Opens a disclosure: a column with its summary row (`Details.summary
    /// [open]`, a chevron `Details.marker` pointing to the inline end while
    /// closed and down while open) and a content column `<id>/Content`
    /// (`Details.content`) that is hidden while closed. Build the content,
    /// then call [`Ui::close_details`]. Role button with `expanded` on the
    /// summary, group on the whole. Probe: `<id>/Summary`, `<id>/Content`.
    pub fn open_details(&mut self, id: impl Into<Id>, desc: &DetailsDesc) -> DetailsResult {
        let t = self.env.theme;
        let mut open = desc.open;
        self.open(
            id,
            Element {
                direction: Direction::Column,
                width: desc.width.unwrap_or(Size::grow()),
                part: crate::dress::PartRef::parse("details::part(root)", ElementClass::Panel).ok(),
                ..Element::default()
            },
        );
        let summary = if desc.summary.is_empty() {
            "Details"
        } else {
            desc.summary.as_str()
        };
        let ink = self.role("ink");
        let icon = self.by_text(16.0);
        let rtl = self.rtl();
        let built = self.ctl(
            "Summary",
            Ctl {
                part: "details::part(summary)",
                cls: ElementClass::Button,
                flags: vec![open.then_some("open")],
                disabled: desc.disabled,
                width: Some(Size::grow()),
                padding: Insets::xy(t.space.sm, t.space.xs),
                gap: t.space.sm,
                align: Align::Start,
                wash: Some(Some(ink)),
                wash_by: 0.06,
                lift: 0.0,
                press: 0.99,
                semantics: Some(Semantics {
                    role: Some("button".into()),
                    name: Some(summary.to_owned()),
                    expanded: Some(open),
                    ..Semantics::default()
                }),
                ..Ctl::default()
            },
            |ui, _| {
                // Closed it points to the inline end (right, left in RTL),
                // open it points down: HTML's disclosure triangle.
                ui.chevron(
                    "Marker",
                    icon,
                    ink,
                    open,
                    2.2,
                    (if rtl { 1.0 } else { -1.0 }, 0.0),
                );
                ui.text(
                    "Text",
                    summary,
                    Text {
                        width: Some(Size::grow()),
                        ..Text::default()
                    },
                );
            },
        );
        let toggled = built.r.clicked && !desc.disabled;
        if toggled {
            open = !open;
            self.feedback(if open {
                SoundEvent::Open
            } else {
                SoundEvent::Close
            });
        }
        self.open(
            "Content",
            Element {
                direction: Direction::Column,
                width: Size::grow(),
                padding: Insets::xy(t.space.sm, t.space.xs),
                gap: t.space.sm,
                part: crate::dress::PartRef::parse("details::part(content)", ElementClass::Panel)
                    .ok(),
                ..Element::default()
            },
        );
        let content = self.parent();
        self.tree.nodes[content].hidden = !open;
        DetailsResult { open, toggled }
    }

    /// Closes a disclosure opened with [`Ui::open_details`].
    pub fn close_details(&mut self) {
        self.close();
        self.close();
        self.semantics(&Semantics {
            role: Some("group".into()),
            ..Semantics::default()
        });
    }

    /// A disclosure whose content `f` builds (hidden while closed).
    pub fn details(
        &mut self,
        id: impl Into<Id>,
        desc: &DetailsDesc,
        f: impl FnOnce(&mut Ui),
    ) -> DetailsResult {
        let r = self.open_details(id, desc);
        f(self);
        self.close_details();
        r
    }
}
