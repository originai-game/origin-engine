//! One idle renderer per host thread. A lease returns only GPU resources;
//! the next run reconstructs all mutable renderer state before using them.

// napi registers the exports outside `cfg(test)` only.
#![cfg_attr(test, allow(dead_code))]

use napi_derive::napi;
use og_render::{RenderDesc, RenderError, Renderer};
use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};

thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static CLEANUP: Cell<bool> = const { Cell::new(false) };
    static IDLE: RefCell<Option<Renderer>> = const { RefCell::new(None) };
    static INPUT: RefCell<Option<(std::path::PathBuf, std::path::PathBuf)>> = const { RefCell::new(None) };
}

#[napi(js_name = "og_node_input_root")]
pub fn input_root(root: Option<String>, frozen: Option<String>) {
    INPUT.with(|input| *input.borrow_mut() = root.zip(frozen).map(|(a, b)| (a.into(), b.into())));
}

pub(crate) fn asset_file(path: &str) -> String {
    INPUT.with(|input| {
        input
            .borrow()
            .as_ref()
            .and_then(|(root, frozen)| {
                std::path::Path::new(path)
                    .strip_prefix(root)
                    .ok()
                    .map(|rel| frozen.join(rel))
            })
            .map_or_else(
                || path.to_owned(),
                |path| path.to_string_lossy().into_owned(),
            )
    })
}

#[napi(js_name = "og_node_resident")]
pub fn resident(env: napi::Env, enabled: bool) -> napi::Result<()> {
    if enabled && !CLEANUP.get() {
        env.add_env_cleanup_hook((), |_| {
            ENABLED.set(false);
            IDLE.with(|idle| idle.borrow_mut().take());
            CLEANUP.set(false);
        })?;
        CLEANUP.set(true);
    }
    ENABLED.set(enabled);
    if !enabled {
        IDLE.with(|idle| idle.borrow_mut().take());
    }
    Ok(())
}

pub(crate) struct Lease(Option<Renderer>);

impl Lease {
    pub(crate) fn new(desc: &RenderDesc) -> Result<Self, RenderError> {
        let idle = ENABLED
            .get()
            .then(|| IDLE.with(|r| r.borrow_mut().take()))
            .flatten();
        match idle {
            Some(renderer) => renderer.restart_blocking(desc),
            None => Renderer::new_blocking(desc),
        }
        .map(|renderer| Self(Some(renderer)))
    }
}

impl From<Renderer> for Lease {
    fn from(renderer: Renderer) -> Self {
        Self(Some(renderer))
    }
}

impl Deref for Lease {
    type Target = Renderer;

    fn deref(&self) -> &Renderer {
        self.0.as_ref().expect("live renderer lease")
    }
}

impl DerefMut for Lease {
    fn deref_mut(&mut self) -> &mut Renderer {
        self.0.as_mut().expect("live renderer lease")
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        if ENABLED.try_with(Cell::get).unwrap_or(false) {
            let _ = IDLE.try_with(|idle| *idle.borrow_mut() = self.0.take());
        }
    }
}
