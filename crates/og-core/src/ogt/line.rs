/// One line a format takes.
#[derive(Clone, Copy, Debug)]
pub struct Line {
    pub key: &'static str,
    /// What follows the key, as written: `<x> <y> <z>`, `true | false`.
    pub args: &'static str,
    /// The value it has when the file leaves it out, as the line's
    /// arguments; empty when leaving it out means none or depends.
    pub default: &'static str,
    pub doc: &'static str,
    /// For a vfx element's line, the elements that take it (`ring` is a
    /// `mesh ring`); empty when every element takes it.
    pub within: &'static [&'static str],
    /// The line (or block) as a file writes it.
    pub example: &'static str,
}

impl Line {
    /// A line of a file's body.
    pub const fn new(
        key: &'static str,
        args: &'static str,
        default: &'static str,
        doc: &'static str,
        example: &'static str,
    ) -> Line {
        Line {
            key,
            args,
            default,
            doc,
            within: &[],
            example,
        }
    }

    /// A line of the elements `within` (every element when empty).
    pub const fn of(
        within: &'static [&'static str],
        key: &'static str,
        args: &'static str,
        default: &'static str,
        doc: &'static str,
        example: &'static str,
    ) -> Line {
        Line {
            key,
            args,
            default,
            doc,
            within,
            example,
        }
    }
}
