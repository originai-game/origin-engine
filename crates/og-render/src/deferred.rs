//! GPU objects made the first time a frame uses them.
//!
//! Every pipeline is a program to compile, and on WebGL2 wgpu compiles and
//! links each one synchronously when it is made (ANGLE's compile, then the
//! status read that waits for it): the renderer making all of its fixed
//! pipelines up front cost the WebGL2 page 400 to 550 ms before its first
//! frame (examples/WEB-FINDINGS.md #11), most of them for passes the first
//! frame does not draw (water, metaballs, fluids, terrain, scatter,
//! particles, 2D lights, ribbons, bloom, depth of field…). A [`Deferred`]
//! pipeline is made from the same descriptor when a pass first sets it, so
//! the first frame compiles what it draws and nothing else, and every frame
//! is the frame it was. [`Renderer::warm`](crate::Renderer::warm) makes
//! the rest one at a time when the host has idle time (behind the web
//! page's play gate), so a pass met later does not stop a frame to compile.

use std::cell::{Cell, OnceCell};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

/// A value made by its closure on first use (deref), at most once.
pub(crate) struct Deferred<T> {
    value: OnceCell<T>,
    make: Cell<Option<Box<dyn FnOnce() -> T>>>,
}

impl<T> Deferred<T> {
    pub(crate) fn new(make: impl FnOnce() -> T + 'static) -> Deferred<T> {
        Deferred {
            value: OnceCell::new(),
            make: Cell::new(Some(Box::new(make))),
        }
    }

    /// The value, made now if it was not yet.
    pub(crate) fn get(&self) -> &T {
        self.value.get_or_init(|| {
            let make = self
                .make
                .take()
                .expect("a deferred value is made once, by its first use");
            make()
        })
    }

    /// Whether the value has been made.
    pub(crate) fn made(&self) -> bool {
        self.value.get().is_some()
    }
}

impl<T> Deref for Deferred<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
    }
}

impl<T> DerefMut for Deferred<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.get();
        self.value.get_mut().expect("made just now")
    }
}

/// A shader module parsed when the first pipeline that uses it is made.
pub(crate) type Module = Rc<Deferred<wgpu::ShaderModule>>;

/// A WGSL module made on first use ([`Module`]).
pub(crate) fn module(device: &wgpu::Device, label: &'static str, source: String) -> Module {
    let device = device.clone();
    Rc::new(Deferred::new(move || {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        })
    }))
}

/// A shader module a deferred pipeline is made from: one already made, or a [`Module`].
pub(crate) trait AsModule {
    fn module(&self) -> &wgpu::ShaderModule;
}

impl AsModule for wgpu::ShaderModule {
    fn module(&self) -> &wgpu::ShaderModule {
        self
    }
}

impl AsModule for Module {
    fn module(&self) -> &wgpu::ShaderModule {
        self.get()
    }
}

/// Something [`Renderer::warm`](crate::Renderer::warm) can make ahead of use.
pub(crate) trait Warm {
    /// Make it now if it is not made; true when this call made it.
    fn warm(&self) -> bool;
}

impl<T> Warm for Deferred<T> {
    fn warm(&self) -> bool {
        if self.made() {
            return false;
        }
        self.get();
        true
    }
}

/// Makes the first of `items` not yet made; false when all are.
pub(crate) fn warm_one(items: &[&dyn Warm]) -> bool {
    items.iter().any(|w| w.warm())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn made_once_on_first_use_and_warmed_in_order() {
        let made = Rc::new(Cell::new(0));
        let m = made.clone();
        let a = Deferred::new(move || {
            m.set(m.get() + 1);
            7
        });
        assert!(!a.made());
        assert_eq!(made.get(), 0);
        assert_eq!(*a, 7);
        assert_eq!(*a.get(), 7);
        assert_eq!(made.get(), 1);
        let b = Deferred::new(|| "b");
        let c = Deferred::new(|| "c");
        assert!(warm_one(&[&a, &b, &c]));
        assert!(b.made() && !c.made());
        assert!(warm_one(&[&a, &b, &c]));
        assert!(!warm_one(&[&a, &b, &c]));
        let mut d = Deferred::new(|| vec![1]);
        d.push(2);
        assert_eq!(*d, vec![1, 2]);
    }
}
