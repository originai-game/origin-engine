//! The documents' extension traits on the core types: what the document
//! build adds to the UI, its documents' state and the retained host.

pub(crate) use crate::doc::build::color::ColorUi;
pub(crate) use crate::doc::build::datetime::DatetimeUi;
pub(crate) use crate::doc::build::generated::{
    GeneratedDocs, GeneratedStyle, GeneratedUi, GeneratedUi2,
};
pub(crate) use crate::doc::build::gestures::GesturesUi;
pub(crate) use crate::doc::build::kbd::KbdUi;
pub(crate) use crate::doc::build::media::MediaUi;
pub(crate) use crate::doc::build::mount::{MountDocs, MountUi2};
pub use crate::doc::build::mount::{MountUi, MountUi3};
pub(crate) use crate::doc::build::nodes::events::NodesEventsUi;
pub(crate) use crate::doc::build::nodes::lists::{NodesListsDocs, NodesListsUi, NodesListsUi2};
pub(crate) use crate::doc::build::nodes::{NodesDocs, NodesUi};
pub(crate) use crate::doc::build::popover::PopoverUi;
pub(crate) use crate::doc::build::retained::RetainedRetainedHost;
pub(crate) use crate::doc::build::retained::compile::RetainedCompileRetainedHost;
pub(crate) use crate::doc::build::retained::content::RetainedContentRetainedHost;
pub(crate) use crate::doc::build::retained::emit::RetainedEmitRetainedHost;
pub(crate) use crate::doc::build::retained::frame::RetainedFrameRetainedHost;
pub use crate::doc::build::retained::mounting::RetainedMountingUi;
pub(crate) use crate::doc::build::retained::mounting::RetainedMountingUi2;
pub(crate) use crate::doc::build::retained::pointer::RetainedPointerRetainedHost;
pub(crate) use crate::doc::build::retained::queries::RetainedQueriesRetainedHost;
pub(crate) use crate::doc::build::retained::records::RetainedRecordsRetainedHost;
pub(crate) use crate::doc::build::retained::refresh::RetainedRefreshRetainedHost;
pub(crate) use crate::doc::build::retained::registration::{
    RetainedRegistrationDocs, RetainedRegistrationUi,
};
pub(crate) use crate::doc::build::retained::selection::RetainedSelectionRetainedHost;
pub(crate) use crate::doc::build::retained::structural::RetainedStructuralRetainedHost;
pub(crate) use crate::doc::build::retained::virtualized::RetainedVirtualizedRetainedHost;
pub(crate) use crate::doc::build::svg::elements::SvgElementsUi;
pub(crate) use crate::doc::build::text_motion::TextMotionUi;
pub use crate::doc::build::touch::TouchUi;
pub(crate) use crate::doc::build::touch::{TouchTree, TouchUi2};
pub(crate) use crate::doc::build::viewport::ViewportUi;
pub(crate) use crate::doc::build::world::WorldUi;
pub use crate::doc::svelte::{SvelteUiDocument, SvelteUiDocument2};
pub use crate::layers::LayersUi;
pub(crate) use crate::layers::LayersUi2;
pub use crate::probe::ProbeUi;
pub use crate::style_pack::StylePackUi;
