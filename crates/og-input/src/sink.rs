//! The input layer's own path: [`InputSink`], which the probe's `input`
//! member, the command channel and the host's input ring deliver through, and
//! [`InputLayer`], the engine's sink over the `og.Input` resource.

use crate::event::InjectedInput;
use crate::resource::{register, Input};
use crate::ring;
use og_ecs::World;

/// The input layer's own path. The same shape as og-probe's interim
/// `InputSink`, so the probe can take an [`InputLayer`] once it re-exports
/// this trait.
pub trait InputSink {
    /// Called once when the probe (or host) is created: register what the
    /// sink writes.
    fn attach(&mut self, world: &mut World) -> og_core::Result<()>;
    /// Deliver one event between ticks. `false` is a refusal: the layer
    /// cannot take this event (an unknown key code, a missing resource).
    fn inject(&mut self, world: &mut World, event: &InjectedInput) -> bool;
    /// Called after every tick: expire edges.
    fn end_tick(&mut self, world: &mut World);
    /// Events the input layer accepted since load.
    fn accepted(&self) -> u32;
}

/// The engine's [`InputSink`]: writes the world's [`Input`] resource and
/// sends `og.KeyDown`, `og.KeyUp`, `og.PointerDown`, `og.PointerUp`,
/// `og.PointerMove` and `og.Wheel` (see [`Input::apply`] for the rules).
/// Everything it does is world state (the resource and the event queues), so
/// injected input is in snapshots and the world hash and replays exactly.
///
/// Order: events reach the world in the order they are injected, and the
/// events a tick's end raises follow in key-code order, then buttons. The
/// host drains its ring once per frame before the frame's ticks, oldest
/// first; probe and command input is injected where its call runs.
#[derive(Debug, Default)]
pub struct InputLayer {
    accepted: u32,
}

/// The name og-probe's interim default sink has; the same type.
pub type WorldInput = InputLayer;

/// What [`InputLayer::drain_ring`] did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RingReport {
    /// Events taken from the ring and accepted.
    pub accepted: u32,
    /// Events the input layer refused (an unknown key code).
    pub refused: u32,
    /// Slots that did not decode, with the reason.
    pub skipped: Vec<String>,
    /// The new read index: store it in the ring's header word 1.
    pub read: u32,
}

impl InputLayer {
    pub fn new() -> InputLayer {
        InputLayer::default()
    }

    /// Injects every event a ring holds (see [`ring::drain`]), oldest first.
    /// The caller stores [`RingReport::read`] back into the ring.
    pub fn drain_ring(&mut self, world: &mut World, ring: &[u8]) -> RingReport {
        let drained = ring::drain(ring);
        let mut report = RingReport {
            skipped: drained.skipped,
            read: drained.read,
            ..RingReport::default()
        };
        for event in &drained.events {
            if self.inject(world, event) {
                report.accepted += 1;
            } else {
                report.refused += 1;
            }
        }
        report
    }
}

impl InputSink for InputLayer {
    fn attach(&mut self, world: &mut World) -> og_core::Result<()> {
        register(world)?;
        if !world.has_resource::<Input>() {
            world.insert_resource(Input::default())?;
        }
        Ok(())
    }

    fn inject(&mut self, world: &mut World, event: &InjectedInput) -> bool {
        let Ok(input) = world.resource_mut::<Input>() else {
            return false;
        };
        let Some(events) = input.apply(event) else {
            return false;
        };
        for e in events {
            if e.send(world).is_err() {
                return false;
            }
        }
        self.accepted += 1;
        true
    }

    fn end_tick(&mut self, world: &mut World) {
        let Ok(input) = world.resource_mut::<Input>() else {
            return;
        };
        for e in input.end_tick() {
            // The event types are registered by attach; a world without them
            // has no input layer attached and nothing to tell.
            let _ = e.send(world);
        }
    }

    fn accepted(&self) -> u32 {
        self.accepted
    }
}
