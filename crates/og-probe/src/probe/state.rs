//! [`Probe`]'s states: its fields grouped by the subject that owns them.

use super::*;

/// What the host gave the probe: its input sink, renderer, UI and files,
/// and the hooks its members answer through.
pub(crate) struct Host {
    pub(crate) engine: String,
    pub(crate) input: Box<dyn InputSink>,
    pub(crate) capture: Option<Box<dyn Capture>>,
    pub(crate) ui: Option<Box<dyn UiTree>>,
    pub(crate) reloader: Option<Box<dyn Reloader>>,
    pub(crate) facts: Option<FactsFn>,
    /// The frame as data and picking ([`Probe::set_scene_view`]).
    pub(crate) scene_view: Option<Box<dyn SceneView>>,
    /// The boot scene's write-back ([`Probe::set_scene_writer`]).
    pub(crate) scene_writer: Option<Box<dyn SceneWriter>>,
    /// Reference files for `expect:frame` ([`Probe::set_reader`]).
    pub(crate) reader: Option<ReaderFn>,
    pub(crate) listen: Option<ListenFn>,
    pub(crate) memory: Option<MemoryFn>,
    pub(crate) derive: Option<DeriveFn>,
    pub(crate) register: Option<RegisterFn>,
}
