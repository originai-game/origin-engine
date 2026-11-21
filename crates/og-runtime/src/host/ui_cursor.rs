//! The pointer's shape a host shows (CSS UI 4 §5.1): the `cursor-*` of
//! the TSX element under the pointer, as `og.ui.frameCursor` answers it.
//! The web shell sets the canvas's CSS cursor from it; Origin Desktop's
//! shared frame hears it as its `Cursor` message (og-render's
//! `shared_frames`).

use std::rc::Rc;

use crate::engine::Engine;

/// A cursor image ready to show.
#[derive(Debug, PartialEq)]
pub struct CursorPixels {
    pub width: u32,
    pub height: u32,
    /// Row-major RGBA8, sRGB, straight alpha.
    pub rgba: Vec<u8>,
    /// The point of the image that is the pointer's position, image px.
    pub hotspot: [u32; 2],
    /// Its source (the texture, the region and the hotspot), so a host
    /// tells a new image from the one it shows without comparing pixels.
    pub id: u64,
}

/// The pointer's shape this frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CursorShape {
    /// The CSS keyword (`default` for `auto`), shown when `image` is none
    /// and where the image cannot be.
    pub keyword: &'static str,
    pub image: Option<Rc<CursorPixels>>,
}

impl CursorShape {
    /// What tells this shape from another: its keyword and its image's id.
    pub fn id(&self) -> (&'static str, u64) {
        (self.keyword, self.image.as_ref().map_or(0, |i| i.id))
    }
}

/// The UI's cursor this frame: none while a pad's virtual cursor shows
/// (the style draws it), else the TSX element's under the pointer, the
/// arrow for `auto`.
pub(crate) fn shape(eng: &Engine) -> CursorShape {
    use og_ui_next::Cursor;
    let keyword = if crate::present::ui_next::virtual_cursor(eng.id).is_some_and(|(_, shown)| shown)
    {
        Cursor::None
    } else {
        match crate::present::ui_next::cursor(eng.id).unwrap_or_default() {
            Cursor::Auto => Cursor::Default,
            c => c,
        }
    };
    CursorShape {
        keyword: keyword.name(),
        image: None,
    }
}
