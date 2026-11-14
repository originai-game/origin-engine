//! The documents' state the stages read: the documents a UI knows, what
//! their mounts keep between ticks, and what a build tells code and the probe.

use super::*;

/// A text field's constraint validity as last built ([`Act::Submit`]).
#[derive(Clone, Debug)]
pub struct FieldValidity {
    pub mount: String,
    pub doc: String,
    pub node: usize,
    pub row_key: String,
    pub invalid: bool,
}

/// Something a document did that code may want: a click, a changed
/// control, a named event (`emit`, a timeline's callback), an action, a
/// two-way write (`set`), a Back the layer turned into an event.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocEvent {
    /// The document's mount: the screen's name, or a component instance's path.
    pub mount: String,
    /// The element's path.
    pub path: String,
    /// Stable key of the innermost each row; empty outside a row.
    pub row_key: String,
    /// `click`, `changed`, `select`, `emit`, `action`, `set`, `back`.
    pub kind: String,
    /// The event's or the action's name, or the path a `set` wrote.
    pub name: String,
    pub value: Value,
}

impl DocEvent {
    pub fn to_json(&self) -> Value {
        json!({ "mount": self.mount, "path": self.path, "rowKey": self.row_key, "kind": self.kind, "name": self.name, "value": self.value })
    }
}

/// A probe edit of a prop (`ui:set`): presentation only, marked in the dump.
#[derive(Clone, Debug, PartialEq)]
pub struct Edit {
    pub path: String,
    pub prop: String,
    pub spec: usize,
    pub value: PropValue,
    /// The value as given.
    pub source: String,
}

/// One element a document built this tick, for the probe.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Built {
    pub path: String,
    /// The document's name and the node's line.
    pub doc: String,
    pub node: usize,
    pub line: usize,
    /// The mount it belongs to (a screen's name is its layer).
    pub mount: String,
    /// `(prop, expression as written, value now)`.
    pub bindings: Vec<(String, String, Value)>,
    /// Its probe edits: `(prop, value as given)`.
    pub edits: Vec<(String, String)>,
    /// The state groups it declares, with the state each is in by its binding.
    pub states: Vec<(String, String)>,
}

/// What one retained element registered as it was emitted: its probe
/// record, render fields and art slots. It lives with the element and is
/// replaced only when the element is emitted again.
#[derive(Clone, Debug, Default)]
pub struct DocRecord {
    pub built: Vec<Built>,
    pub render_fields: Vec<(u64, super::fields::RenderFields)>,
    pub arts: Vec<(String, crate::slot::SlotSpec)>,
}

/// A view transition's covers still opening after their layer left: the
/// covers, their colour and image, and the tick it left.
pub type Revealing = (
    Vec<crate::motion::css::CssAnimation>,
    Option<crate::Srgba>,
    Option<String>,
    u64,
);

/// What the documents keep between ticks that a snapshot carries: their
/// mounts, the reactive session, the elements seen, the triggers' last
/// values and the states they set. The documents hold it in their state
/// ([`super::state::DocState`]); the stages read and write it in
/// [`Ui::save_state`] and [`Ui::restore_state`].
#[derive(Clone, Debug, Default)]
pub struct DocMemory {
    pub retained: super::instances::RetainedHost,
    pub reactive: crate::reactive::session::Session,
    /// Document elements seen last tick (for `on appear`).
    pub seen: BTreeSet<u64>,
    pub seen_now: BTreeSet<u64>,
    /// A trigger's last value (`on change`), by element key and trigger.
    pub last: BTreeMap<(u64, usize), Value>,
    /// The states triggers set (`on click state selection on`), by element key and group.
    pub states: BTreeMap<(u64, String), String>,
}

impl DocMemory {
    /// The tick starts: what was seen is last tick's, and the mounts count afresh.
    pub fn begin(&mut self) {
        self.seen = std::mem::take(&mut self.seen_now);
        self.retained.evaluated = 0;
        self.retained.resolved = 0;
        self.retained.reconciled = 0;
    }

    /// Restores what a snapshot carried ([`Ui::restore_state`]).
    pub fn restore(&mut self, v: &Value) -> Result<(), String> {
        let retained = self.restore_document_mounts(&v["mounts"], &v["mountRegistrations"])?;
        if let Some(stores) = v.get("stores") {
            self.reactive.restore(stores)?;
        } else {
            self.reactive = Default::default();
        }
        self.reactive.graph.clear_bindings();
        self.retained = retained;
        let key = |v: &Value| v.as_str().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        self.seen_now = v["seen"]
            .as_array()
            .map(|a| a.iter().map(key).collect())
            .unwrap_or_default();
        self.seen = self.seen_now.clone();
        self.last = v["last"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|s| {
                        (
                            (key(&s[0]), s[1].as_u64().unwrap_or(0) as usize),
                            s[2].clone(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.states = v["states"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|s| {
                        (
                            (key(&s[0]), s[1].as_str().unwrap_or_default().to_string()),
                            s[2].as_str().unwrap_or_default().to_string(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(())
    }

    /// The art slot `path`'s spec among the mounted elements' records.
    pub fn record_art_slot_mut(&mut self, path: &str) -> Option<&mut crate::slot::SlotSpec> {
        self.retained
            .record_mut(crate::id::key_of_path(path))
            .and_then(|r| r.arts.iter_mut().find(|(p, _)| p == path))
            .map(|(_, spec)| spec)
    }
}

/// What the controls naming a popover last did to it, by its key
/// ([`popover_key`]): presentation, rebuilt from the controls as they build.
#[derive(Clone, Copy, Debug, Default)]
pub struct PopoverLink {
    /// The control it is placed against (its node index and key).
    pub invoker: (usize, u64),
    /// An `interestfor` control holds the player's interest.
    pub interest: bool,
    /// Its layer's key, as last built (hovering it keeps a hint shown).
    pub layer: u64,
    /// It showed when last built (`ontoggle` fires as that changes).
    pub shown: bool,
    /// `auto`, `manual` or `hint`, as last built.
    pub mode: Mode,
    /// Opened by `contextmenu`: the point it opened at, its anchor.
    pub point: Option<[f32; 2]>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Auto,
    Manual,
    Hint,
}

impl Ui {
    /// A document by name.
    pub fn document(&self, name: &str) -> Option<&UiDocument> {
        self.docs.library.documents.get(name).map(|d| &**d)
    }

    /// Every document's name.
    pub fn documents(&self) -> Vec<String> {
        self.docs.library.documents.keys().cloned().collect()
    }
}

impl crate::ui::Docs {
    /// A document's `-og-material` as the layer it draws: a built-in's, or
    /// a defined asset's with its defaults under the numbers written.
    pub fn css_material(
        &self,
        m: crate::dress::DressMaterial,
    ) -> Result<crate::paint::Layer, String> {
        let Some(path) = m.asset else {
            return Ok(m.layer);
        };
        let (handle, mut params) = *self.library.materials.get(&path).ok_or_else(|| {
            format!("-og-material: \"{path}\" is not a loaded UI material (a `type ui-material` asset under assets/)")
        })?;
        for k in 0..m.params {
            params[k / 4][k % 4] = m.layer.params[k / 4][k % 4];
        }
        Ok(crate::paint::Layer {
            material: crate::paint::Material::Custom(handle),
            params,
            filter: None,
        })
    }

    pub(crate) fn last_art_path(&self) -> String {
        self.last_art_entry().0.clone()
    }

    pub fn last_art_entry(&self) -> &(String, crate::slot::SlotSpec) {
        self.state
            .last_art_entry()
            .expect("an art slot was registered")
    }

    /// The art slot `path`'s spec, where it was registered.
    pub(crate) fn art_slot_mut(&mut self, path: &str) -> Option<&mut crate::slot::SlotSpec> {
        self.state.art_slot_mut(path)
    }
}

impl Ui {
    /// The tick's document bookkeeping starts (from `begin`).
    pub(crate) fn docs_begin(&mut self) {
        self.docs.state.begin_tick();
        let c = &mut self.docs.containers;
        // Last tick's containers stay watched.
        let mut live = std::mem::take(&mut c.built);
        live.extend(self.docs.state.memory.retained.containers());
        self.watch_document_containers(live);
        self.docs.sync_scroll_states(&self.interaction, &self.tree);
    }
}

impl crate::ui::Docs {
    /// Plays the Rive artboard `id` in element `node` (its key `key`) this
    /// tick: the element's player goes on one tick, a restored one is
    /// rebuilt from its bytes, a new one starts; the element draws it over
    /// its background ([`crate::paint::ElementDraw::rive`]).
    #[allow(clippy::too_many_arguments)]
    pub fn play_rive(
        &mut self,
        node: usize,
        id: &str,
        path: &str,
        env: &crate::ui::Env,
        out: &mut crate::ui::Out,
        interaction: &mut crate::ui::Interaction,
        tree: &mut crate::ui::Tree,
    ) {
        let key = tree.nodes[node].key;
        let Some(board) = self.library.rive.get(id).cloned() else {
            out.warnings.push(format!(
                "{path}: og-rive src=\"{id}\" names no Rive artboard the UI has; og.load.document loads those its document names"
            ));
            return;
        };
        let tick = interaction.state.tick;
        let dt = 1.0 / env.ticks_per_second.max(1) as f32;
        let player = match interaction.state.rive.remove(&key) {
            Some(crate::ui::RiveSlot {
                board: b,
                player: Some(mut p),
                ..
            }) if b == id => {
                if !Arc::ptr_eq(&p.artboard, &board) {
                    // A hot reload: on from where it was, when that fits.
                    let mut q = crate::rive::Player::new(board.clone(), None);
                    p = if q.restore(&p.save()).is_ok() {
                        q
                    } else {
                        crate::rive::Player::new(board, None)
                    };
                }
                p.advance(dt);
                p
            }
            Some(crate::ui::RiveSlot {
                board: b,
                player: None,
                saved,
                ..
            }) if b == id => {
                let mut p = crate::rive::Player::new(board, None);
                if let Err(e) = p.restore(&saved) {
                    out.warnings.push(format!(
                        "{path}: the saved state of Rive artboard {id} does not fit it ({e}); it is kept and not drawn"
                    ));
                    interaction.state.rive.insert(
                        key,
                        crate::ui::RiveSlot {
                            board: b,
                            player: None,
                            saved,
                            built: tick,
                        },
                    );
                    return;
                }
                p.advance(dt);
                p
            }
            _ => crate::rive::Player::new(board, None),
        };
        tree.draws.insert(
            node,
            crate::paint::ElementDraw {
                rive: Some(crate::paint::RiveDraw(player.clone())),
                ..crate::paint::ElementDraw::default()
            },
        );
        tree.dressable.remove(&node);
        interaction.state.rive.insert(
            key,
            crate::ui::RiveSlot {
                board: id.to_owned(),
                player: Some(player),
                saved: Vec::new(),
                built: tick,
            },
        );
    }
}
