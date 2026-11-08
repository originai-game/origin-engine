//! Elements a document names that HTML has no core name for (SVG, a
//! viewport, a joystick…), attached by the document runtime: a tag the
//! core table does not know comes here first.

/// The internal kind lowercase `tag` names (one [`super::schema_extra`]
/// registers), inside an element whose selector tag is `parent_tag`.
/// `Some(Err(hint))`: the tag is known but not valid there. `None`: not
/// an extern element.
pub(super) fn kind(tag: &str, parent_tag: Option<&str>) -> Option<Result<&'static str, String>> {
    let _ = (tag, parent_tag);
    None
}
