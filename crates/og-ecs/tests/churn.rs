//! 10,000-entity structural churn (spawn, despawn, add, remove, overwrite)
//! against a shadow model: after every phase the world holds exactly the
//! model's entities and values, queries visit each matching entity once with
//! the right column values, and dead ids stay dead.

use og_core::math::{Quat, Vec3};
use og_core::{ErrorKind, Rng};
use og_ecs::{Entity, FieldRef, QueryDesc, Reflect, SchemaId, World, WorldDesc};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Pos {
    p: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Vel {
    v: Vec3,
    spin: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Hp {
    hp: i32,
    armor: u32,
}

/// 8- and 16-byte-aligned columns.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Big {
    id: u64,
    mass: f64,
    q: Quat,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
struct Tag;

#[derive(Clone, Debug, Default, PartialEq)]
struct Model {
    pos: Option<Pos>,
    vel: Option<Vel>,
    hp: Option<Hp>,
    big: Option<Big>,
    tag: bool,
}

struct Fields {
    pos_p: FieldRef,
    vel_v: FieldRef,
    vel_spin: FieldRef,
    hp_hp: FieldRef,
    hp_armor: FieldRef,
    big_id: FieldRef,
    big_mass: FieldRef,
    big_q: FieldRef,
}

struct Harness {
    world: World,
    model: BTreeMap<Entity, Model>,
    live: Vec<Entity>,
    dead: Vec<Entity>,
    rng: Rng,
    ids: [SchemaId; 5],
    f: Fields,
}

impl Harness {
    fn new(seed: u32) -> Harness {
        let mut world = World::new(&WorldDesc::default()).unwrap();
        let ids = [
            world.register::<Pos>().unwrap(),
            world.register::<Vel>().unwrap(),
            world.register::<Hp>().unwrap(),
            world.register::<Big>().unwrap(),
            world.register::<Tag>().unwrap(),
        ];
        let f = Fields {
            pos_p: world.field::<Pos>("p").unwrap(),
            vel_v: world.field::<Vel>("v").unwrap(),
            vel_spin: world.field::<Vel>("spin").unwrap(),
            hp_hp: world.field::<Hp>("hp").unwrap(),
            hp_armor: world.field::<Hp>("armor").unwrap(),
            big_id: world.field::<Big>("id").unwrap(),
            big_mass: world.field::<Big>("mass").unwrap(),
            big_q: world.field::<Big>("q").unwrap(),
        };
        Harness {
            world,
            model: BTreeMap::new(),
            live: Vec::new(),
            dead: Vec::new(),
            rng: Rng::new(seed),
            ids,
            f,
        }
    }

    fn f32(&mut self) -> f32 {
        self.rng.range_f32(-100.0, 100.0)
    }
    fn pos(&mut self) -> Pos {
        Pos {
            p: Vec3::new(self.f32(), self.f32(), self.f32()),
        }
    }
    fn vel(&mut self) -> Vel {
        Vel {
            v: Vec3::new(self.f32(), self.f32(), self.f32()),
            spin: self.f32(),
        }
    }
    fn hp(&mut self) -> Hp {
        Hp {
            hp: self.rng.integer(-1000, 1000) as i32,
            armor: self.rng.integer(0, 1 << 30) as u32,
        }
    }
    fn big(&mut self) -> Big {
        let id = (u64::from(self.rng.next_u32()) << 32) | u64::from(self.rng.next_u32());
        Big {
            id,
            mass: self.rng.next_f64() * 1e12,
            q: Quat::from_xyzw(self.f32(), self.f32(), self.f32(), self.f32()),
        }
    }

    fn pick(&mut self) -> Option<usize> {
        if self.live.is_empty() {
            None
        } else {
            Some(self.rng.integer(0, self.live.len() as i64 - 1) as usize)
        }
    }

    fn spawn(&mut self) {
        let mut m = Model::default();
        let e = match self.rng.integer(0, 3) {
            0 => {
                let e = self.world.spawn(()).unwrap();
                if self.rng.next_f64() < 0.5 {
                    let v = self.pos();
                    self.world.insert(e, v).unwrap();
                    m.pos = Some(v);
                }
                if self.rng.next_f64() < 0.5 {
                    let v = self.hp();
                    self.world.insert(e, v).unwrap();
                    m.hp = Some(v);
                }
                if self.rng.next_f64() < 0.3 {
                    self.world.insert(e, Tag).unwrap();
                    m.tag = true;
                }
                e
            }
            1 => {
                let (p, v) = (self.pos(), self.vel());
                m.pos = Some(p);
                m.vel = Some(v);
                self.world.spawn((p, v)).unwrap()
            }
            2 => {
                let (h, b) = (self.hp(), self.big());
                m.hp = Some(h);
                m.big = Some(b);
                m.tag = true;
                self.world.spawn((h, b, Tag)).unwrap()
            }
            _ => {
                let (b, v, p, h) = (self.big(), self.vel(), self.pos(), self.hp());
                m = Model {
                    pos: Some(p),
                    vel: Some(v),
                    hp: Some(h),
                    big: Some(b),
                    tag: true,
                };
                // Bundle order does not matter.
                self.world.spawn((Tag, b, v, p, h)).unwrap()
            }
        };
        assert!(self.model.insert(e, m).is_none(), "{e} handed out twice");
        self.live.push(e);
    }

    fn despawn(&mut self) {
        let Some(i) = self.pick() else { return };
        let e = self.live.swap_remove(i);
        self.world.despawn(e).unwrap();
        self.model.remove(&e);
        self.dead.push(e);
    }

    fn insert(&mut self) {
        let Some(i) = self.pick() else { return };
        let e = self.live[i];
        match self.rng.integer(0, 4) {
            0 => {
                let v = self.pos();
                self.world.insert(e, v).unwrap();
                self.model.get_mut(&e).unwrap().pos = Some(v);
            }
            1 => {
                let v = self.vel();
                self.world.insert(e, v).unwrap();
                self.model.get_mut(&e).unwrap().vel = Some(v);
            }
            2 => {
                let v = self.hp();
                self.world.insert(e, v).unwrap();
                self.model.get_mut(&e).unwrap().hp = Some(v);
            }
            3 => {
                let v = self.big();
                self.world.insert(e, v).unwrap();
                self.model.get_mut(&e).unwrap().big = Some(v);
            }
            _ => {
                self.world.insert(e, Tag).unwrap();
                self.model.get_mut(&e).unwrap().tag = true;
            }
        }
    }

    fn remove(&mut self) {
        let Some(i) = self.pick() else { return };
        let e = self.live[i];
        let m = self.model.get_mut(&e).unwrap();
        let (result, had) = match self.rng.integer(0, 4) {
            0 => (self.world.remove::<Pos>(e), m.pos.take().is_some()),
            1 => (self.world.remove::<Vel>(e), m.vel.take().is_some()),
            2 => (self.world.remove::<Hp>(e), m.hp.take().is_some()),
            3 => (self.world.remove::<Big>(e), m.big.take().is_some()),
            _ => (self.world.remove::<Tag>(e), std::mem::take(&mut m.tag)),
        };
        if had {
            result.unwrap();
        } else {
            assert_eq!(result.unwrap_err().kind(), ErrorKind::NotFound);
        }
    }

    fn poke_the_dead(&mut self) {
        if self.dead.is_empty() {
            return;
        }
        let e = self.dead[self.rng.integer(0, self.dead.len() as i64 - 1) as usize];
        assert_eq!(
            self.world.insert(e, Tag).unwrap_err().kind(),
            ErrorKind::DeadHandle
        );
        assert_eq!(
            self.world.remove::<Tag>(e).unwrap_err().kind(),
            ErrorKind::DeadHandle
        );
        assert_eq!(
            self.world.despawn(e).unwrap_err().kind(),
            ErrorKind::DeadHandle
        );
        assert_eq!(
            self.world.get::<Pos>(e).unwrap_err().kind(),
            ErrorKind::DeadHandle
        );
    }

    fn churn(&mut self, ops: usize) {
        for _ in 0..ops {
            let roll = self.rng.integer(0, 9);
            match roll {
                _ if self.live.len() < 8_000 => self.spawn(),
                0..=2 if self.live.len() < 12_000 => self.spawn(),
                0..=4 => self.despawn(),
                5..=6 => self.insert(),
                7..=8 => self.remove(),
                _ => self.poke_the_dead(),
            }
        }
    }

    fn expected_types(&self, m: &Model) -> Vec<SchemaId> {
        let present = [
            m.pos.is_some(),
            m.vel.is_some(),
            m.hp.is_some(),
            m.big.is_some(),
            m.tag,
        ];
        self.ids
            .iter()
            .zip(present)
            .filter(|(_, p)| *p)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Visits a query's views; returns each matched entity once.
    fn visit(
        &self,
        desc: QueryDesc,
        mut check: impl FnMut(&og_ecs::ArchetypeView<'_>, usize, &Model),
    ) -> BTreeSet<Entity> {
        let mut seen = BTreeSet::new();
        for view in self.world.query(&desc).unwrap() {
            for (row, e) in view.entities().iter().enumerate() {
                assert!(seen.insert(*e), "{e} visited twice");
                check(&view, row, &self.model[e]);
            }
        }
        seen
    }

    fn verify(&mut self) {
        let w = &self.world;
        assert_eq!(w.entity_count(), self.model.len());
        assert_eq!(self.live.len(), self.model.len());
        assert_eq!(w.entities(), self.model.keys().copied().collect::<Vec<_>>());
        for (e, m) in &self.model {
            assert!(w.is_alive(*e));
            assert_eq!(w.components_of(*e).unwrap(), self.expected_types(m), "{e}");
            assert_eq!(w.get::<Pos>(*e).ok(), m.pos, "{e}");
            assert_eq!(w.get::<Vel>(*e).ok(), m.vel, "{e}");
            assert_eq!(w.get::<Hp>(*e).ok(), m.hp, "{e}");
            assert_eq!(w.get::<Big>(*e).ok(), m.big, "{e}");
            assert_eq!(w.has::<Tag>(*e).unwrap(), m.tag, "{e}");
        }
        for e in &self.dead {
            assert!(!w.is_alive(*e), "{e} came back");
        }

        let f = &self.f;
        let with = |pick: fn(&Model) -> bool| {
            self.model
                .iter()
                .filter(|(_, m)| pick(m))
                .map(|(e, _)| *e)
                .collect::<BTreeSet<_>>()
        };
        let seen = self.visit(QueryDesc::new().read::<Pos>(), |v, row, m| {
            assert_eq!(v.read::<Vec3>(f.pos_p).unwrap()[row], m.pos.unwrap().p);
        });
        assert_eq!(seen, with(|m| m.pos.is_some()));
        let seen = self.visit(QueryDesc::new().read::<Vel>().read::<Pos>(), |v, row, m| {
            assert_eq!(v.read::<Vec3>(f.vel_v).unwrap()[row], m.vel.unwrap().v);
            assert_eq!(v.read::<f32>(f.vel_spin).unwrap()[row], m.vel.unwrap().spin);
            assert_eq!(v.read::<Vec3>(f.pos_p).unwrap()[row], m.pos.unwrap().p);
        });
        assert_eq!(seen, with(|m| m.vel.is_some() && m.pos.is_some()));
        let seen = self.visit(
            QueryDesc::new().read::<Big>().without::<Tag>(),
            |v, row, m| {
                let b = m.big.unwrap();
                assert_eq!(v.read::<u64>(f.big_id).unwrap()[row], b.id);
                assert_eq!(v.read::<f64>(f.big_mass).unwrap()[row], b.mass);
                assert_eq!(v.read::<Quat>(f.big_q).unwrap()[row], b.q);
            },
        );
        assert_eq!(seen, with(|m| m.big.is_some() && !m.tag));
        let seen = self.visit(QueryDesc::new().read::<Tag>(), |_, _, m| assert!(m.tag));
        assert_eq!(seen, with(|m| m.tag));
        assert_eq!(
            self.visit(QueryDesc::new(), |_, _, _| {}).len(),
            self.model.len()
        );

        // Writing through a query lands in the right rows.
        for view in self.world.query(&QueryDesc::new().write::<Hp>()).unwrap() {
            let mut hp = view.write::<i32>(f.hp_hp).unwrap();
            let mut armor = view.write::<u32>(f.hp_armor).unwrap();
            for (row, e) in view.entities().iter().enumerate() {
                hp[row] += 1;
                armor[row] ^= 0x5a5a;
                let m = self.model.get_mut(e).unwrap().hp.as_mut().unwrap();
                m.hp += 1;
                m.armor ^= 0x5a5a;
            }
        }
        for (e, m) in &self.model {
            assert_eq!(
                self.world.get::<Hp>(*e).ok(),
                m.hp,
                "{e} after a query write"
            );
        }
    }
}

#[test]
fn ten_thousand_entities_survive_structural_churn() {
    let mut h = Harness::new(7);
    for _ in 0..10_000 {
        h.spawn();
    }
    h.verify();
    for _ in 0..8 {
        h.churn(5_000);
        h.verify();
    }
    assert!(h.model.len() >= 8_000);
    assert!(h.dead.len() > 5_000, "only {} despawns", h.dead.len());
    // Every combination of the five components has turned up.
    assert!(
        h.world.archetype_count() >= 20,
        "{} archetypes",
        h.world.archetype_count()
    );

    // Despawned slots are reused, under a new generation.
    let reused = h.model.keys().filter(|e| e.generation() > 1).count();
    assert!(reused > 1_000, "only {reused} reused slots");
}

#[test]
fn a_refused_spawn_leaves_no_trace() {
    let mut h = Harness::new(11);
    for _ in 0..100 {
        h.spawn();
    }
    let before = h.world.world_hash();
    let p = h.pos();
    let err = h.world.spawn((p, Tag, p)).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert!(err.reason().contains("'Pos' twice"), "{}", err.reason());
    #[derive(Clone, Copy, Reflect)]
    struct Unregistered {
        x: f32,
    }
    let err = h.world.spawn((Unregistered { x: 1.0 },)).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::NotFound);
    assert!(err.reason().contains("register"), "{}", err.reason());
    assert_eq!(h.world.entity_count(), 100);
    // No id was reserved: the world is byte-for-byte what it was.
    assert_eq!(h.world.world_hash(), before);
    h.verify();
}
