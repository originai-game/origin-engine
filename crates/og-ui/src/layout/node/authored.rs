//! A node's authored element. Whatever builds the UI writes it, through
//! the tree; layout, which is given the nodes and never the tree, reads it
//! and writes only what it derives from it ([`super::LayoutBox::used`]).
//! Only [`super::Node::new`] makes one.

use super::Element;

/// A node's element as it was authored: read anywhere (it derefs to the
/// [`Element`]), written only with a [`Build`], which the tree hands out.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Authored(Element);

/// Proof that an input is being written by what builds the UI: only
/// [`crate::ui::Tree::build`] makes one, so the layout stage, which holds
/// the nodes without their tree, cannot write an authored element.
#[derive(Clone, Copy)]
pub(crate) struct Build(());

impl Authored {
    pub(super) fn new(el: Element) -> Authored {
        Authored(el)
    }

    /// The element, to write.
    pub(crate) fn write(&mut self, _: Build) -> &mut Element {
        &mut self.0
    }
}

impl std::ops::Deref for Authored {
    type Target = Element;

    fn deref(&self) -> &Element {
        &self.0
    }
}

impl crate::ui::Tree {
    /// The proof a build writes an authored element with, where the node
    /// is already borrowed ([`Authored::write`]).
    pub(crate) fn build(&self) -> Build {
        Build(())
    }

    /// Node `i`'s authored element, to write.
    pub(crate) fn el_mut(&mut self, i: usize) -> &mut Element {
        self.nodes[i].el.write(Build(()))
    }
}
