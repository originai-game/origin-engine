//! A node's authored inputs: its element, content, props and shapes.
//! Whatever builds the UI writes them, through the tree; layout, which is
//! given the nodes and never the tree, reads them and writes only what it
//! derives from them ([`super::LayoutBox`]: the used element, what the
//! variants set, the fit step), and paint writes only its own results
//! ([`super::Placed`]). Only [`super::Node::new`] makes one.

use super::Element;

/// An input of a node as it was authored: read anywhere (it derefs to the
/// value), written only with a [`Build`], which the tree hands out.
///
/// One writer runs as the build ends rather than while it declares:
/// `Layout::prepare_layout_node` normalises a written node's text for
/// layout (a display text under the art's least size takes the body font,
/// uppercase steps are applied, a pixel font's sizes go on its grid). It is
/// part of the build, is handed the tree and writes with its proof.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Authored<T>(T);

/// Proof that an input is being written by what builds the UI: only
/// [`crate::ui::Tree::build`] makes one, so the layout stage, which holds
/// the nodes without their tree, cannot write an authored input.
#[derive(Clone, Copy)]
pub struct Build(());

impl<T> Authored<T> {
    pub(super) fn new(value: T) -> Authored<T> {
        Authored(value)
    }

    /// The value, to write.
    pub fn write(&mut self, _: Build) -> &mut T {
        &mut self.0
    }
}

impl<T> std::ops::Deref for Authored<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl crate::ui::Tree {
    /// The proof a build writes an authored input with, where the node
    /// is already borrowed ([`Authored::write`]).
    pub fn build(&self) -> Build {
        Build(())
    }

    /// Node `i`'s authored element, to write.
    pub fn el_mut(&mut self, i: usize) -> &mut Element {
        self.nodes[i].el.write(Build(()))
    }

    /// Node `i`'s content, to write.
    pub(crate) fn content_mut(&mut self, i: usize) -> &mut super::Content {
        self.nodes[i].content.write(Build(()))
    }

    /// Node `i`'s props, to write.
    pub fn props_mut(&mut self, i: usize) -> &mut crate::element::Props {
        self.nodes[i].props.write(Build(()))
    }

    /// Node `i`'s shapes, to write.
    pub fn shapes_mut(&mut self, i: usize) -> &mut Vec<super::Shape> {
        self.nodes[i].shapes.write(Build(()))
    }
}
