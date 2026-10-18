//! The live tree for the probe, and the system that runs the UI in the tick.

use crate::input::{InputFrame, OgInput};
use crate::ui::{Ui, Viewport};
use og_ecs::{SystemContext, SystemDesc, World};
use og_probe::{UiElement, UiTree};
use std::sync::{Arc, Mutex, MutexGuard};

/// A [`Ui`] shared between the system that builds it (inside the tick), the
/// probe (between ticks) and the renderer (after the tick).
#[derive(Clone, Debug, Default)]
pub struct SharedUi(Arc<Mutex<Ui>>);

impl SharedUi {
    pub fn new(ui: Ui) -> SharedUi {
        SharedUi(Arc::new(Mutex::new(ui)))
    }

    /// The UI; a panic inside a build closure does not lock it out.
    pub fn lock(&self) -> MutexGuard<'_, Ui> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The probe hook: `probe.set_ui(shared.probe())`.
    pub fn probe(&self) -> Box<dyn UiTree> {
        Box::new(UiProbe(self.clone()))
    }

    /// A system that builds the UI once per tick from the world's `og.Input`
    /// (declared as a read): `world.add_system(desc, run)` with the pair this
    /// returns. `viewport` gives the play surface for the tick (the host's
    /// current canvas size, DPR and safe area).
    pub fn system<V, F>(
        &self,
        name: &str,
        viewport: V,
        build: F,
    ) -> (
        SystemDesc,
        impl Fn(&mut SystemContext<'_>) -> og_core::Result<()> + Send + 'static + use<V, F>,
    )
    where
        V: Fn() -> Viewport + Send + 'static,
        F: Fn(&mut Ui, &SystemContext<'_>) -> og_core::Result<()> + Send + 'static,
    {
        let shared = self.clone();
        let desc = SystemDesc::new(name).read::<OgInput>();
        let run = move |ctx: &mut SystemContext<'_>| {
            let input = match ctx.resource::<OgInput>() {
                Ok(i) => InputFrame::read(&*i),
                Err(_) => InputFrame::default(),
            };
            let mut ui = shared.lock();
            ui.begin(input, viewport());
            let built = build(&mut ui, ctx);
            ui.end();
            built
        };
        (desc, run)
    }
}

/// [`UiTree`] over a [`SharedUi`]: the elements of its last completed frame.
#[derive(Clone, Debug)]
pub struct UiProbe(pub SharedUi);

impl UiTree for UiProbe {
    fn elements(&self, _world: &World) -> Vec<UiElement> {
        self.0.lock().frame().elements.clone()
    }
}
