//! Off-mesh links: a jump, a ladder, a door or a teleport joining two
//! points the surface does not.

use super::*;

/// How an agent crosses an off-mesh link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkKind {
    /// Through the air, on an arc.
    Jump = 0,
    /// Climbing, at half its speed.
    Ladder = 1,
    /// Walking through, while it is open.
    Door = 2,
    /// At once.
    Teleport = 3,
}

impl LinkKind {
    pub fn from_index(i: u32) -> LinkKind {
        match i {
            1 => LinkKind::Ladder,
            2 => LinkKind::Door,
            3 => LinkKind::Teleport,
            _ => LinkKind::Jump,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            LinkKind::Jump => "jump",
            LinkKind::Ladder => "ladder",
            LinkKind::Door => "door",
            LinkKind::Teleport => "teleport",
        }
    }
}

/// An off-mesh link: from `start` to `end` (and back while `bidirectional`),
/// each snapped to the polygon under it within a metre.
#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    pub start: Vec3,
    pub end: Vec3,
    pub kind: LinkKind,
    pub bidirectional: bool,
    /// What crossing costs a path, in metres; 0 is the link's length.
    pub cost: f32,
    /// A closed link is never planned across (a shut door).
    pub open: bool,
    /// Seconds a crossing takes; 0 is the kind's own (by the agent's speed; a teleport is instant).
    pub duration: f32,
    /// The game moves the agent across and says when it is over.
    pub game: bool,
}

impl Link {
    /// What crossing costs a path.
    pub fn path_cost(&self) -> f32 {
        if self.cost > 0.0 {
            self.cost
        } else {
            self.start.distance(self.end)
        }
    }

    /// Appends its byte form.
    pub fn write(&self, out: &mut Vec<u8>) {
        for c in [
            self.start.x,
            self.start.y,
            self.start.z,
            self.end.x,
            self.end.y,
            self.end.z,
            self.cost,
            self.duration,
        ] {
            out.extend_from_slice(&c.to_le_bytes());
        }
        out.push(self.kind as u8);
        out.push(
            u8::from(self.bidirectional) | u8::from(self.open) << 1 | u8::from(self.game) << 2,
        );
    }

    /// Reads one link off the front of `b`.
    pub fn read(b: &mut &[u8]) -> Option<Link> {
        if b.len() < 34 {
            return None;
        }
        let f = |i: usize| f32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().unwrap());
        let flags = b[33];
        let l = Link {
            start: Vec3::new(f(0), f(1), f(2)),
            end: Vec3::new(f(3), f(4), f(5)),
            cost: f(6),
            duration: f(7),
            kind: LinkKind::from_index(b[32] as u32),
            bidirectional: flags & 1 != 0,
            open: flags & 2 != 0,
            game: flags & 4 != 0,
        };
        *b = &b[34..];
        Some(l)
    }
}

/// Where a path crosses a link: `points[corner]` is where it starts and
/// `points[corner + 1]` where it lands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathLink {
    pub corner: u32,
    pub link: u32,
    /// Crossed from its end to its start.
    pub reverse: bool,
}

impl NavMesh {
    /// The mesh with `links` as the ones it was built with (the cook's).
    pub fn with_links(mut self, links: Vec<(u32, Link)>) -> NavMesh {
        self.base_links = links.into_iter().collect();
        self.links = self.base_links.clone();
        self.resolve_links();
        self
    }

    /// Its links now, by id.
    pub fn links(&self) -> impl Iterator<Item = (u32, &Link)> {
        self.links.iter().map(|(k, v)| (*k, v))
    }

    /// The links it was built with, by id.
    pub fn base_links(&self) -> impl Iterator<Item = (u32, &Link)> {
        self.base_links.iter().map(|(k, v)| (*k, v))
    }

    pub fn off_mesh_link(&self, id: u32) -> Option<&Link> {
        self.links.get(&id)
    }

    /// The polygons link `id`'s start and end stand on; `None` while either is off the mesh.
    pub fn link_ends(&self, id: u32) -> Option<(PolyRef, PolyRef)> {
        self.link_ends.get(&id).copied()
    }

    /// Sets, moves or removes (`None`) a link; agents on the mesh re-plan.
    /// Whether both its ends are on the mesh.
    pub fn set_link(&mut self, id: u32, link: Option<Link>) -> bool {
        match link {
            Some(l) => {
                self.links.insert(id, l);
            }
            None => {
                self.links.remove(&id);
            }
        }
        self.resolve_links();
        self.version += 1;
        self.link_ends.contains_key(&id)
    }

    /// Opens or closes link `id` (a door); false for an unknown link.
    pub fn set_link_open(&mut self, id: u32, open: bool) -> bool {
        let Some(l) = self.links.get_mut(&id) else {
            return false;
        };
        if l.open != open {
            l.open = open;
            self.version += 1;
        }
        true
    }

    /// Whether its links differ from the ones it was built with (what a save holds).
    pub fn links_moved(&self) -> bool {
        self.links != self.base_links
    }

    /// Replaces every link (a restore): the base ones when `links` is `None`.
    pub fn restore_links(&mut self, links: Option<Vec<(u32, Link)>>) {
        let want: IdMap<u32, Link> = match links {
            Some(l) => l.into_iter().collect(),
            None => self.base_links.clone(),
        };
        if want != self.links {
            self.links = want;
            self.resolve_links();
            self.version += 1;
        }
    }

    /// Snaps every link's ends to the polygons under them.
    pub(in crate::nav::mesh) fn resolve_links(&mut self) {
        self.link_ends.clear();
        self.link_from.clear();
        if self.links.is_empty() {
            return;
        }
        let ext = Vec3::new(1.0, self.field.params.agent_height.max(1.0), 1.0);
        let mut ends = Vec::new();
        for (&id, l) in &self.links {
            if let (Some((s, _)), Some((e, _))) =
                (self.nearest(l.start, ext), self.nearest(l.end, ext))
            {
                ends.push((id, s, e, l.bidirectional));
            }
        }
        for (id, s, e, both) in ends {
            self.link_ends.insert(id, (s, e));
            self.link_from.entry(s).or_default().push((id, false));
            if both {
                self.link_from.entry(e).or_default().push((id, true));
            }
        }
    }
}
