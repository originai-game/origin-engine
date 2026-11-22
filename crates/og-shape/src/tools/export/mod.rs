//! GLB, OBJ and STL from a [`Scene`] of named solids.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::math::{DVec3, Frame, trig};
use crate::mesh::Solid;
use crate::mesh::skin::Mat4;

use crate::tools::image::Texture;
use crate::tools::json::{Json, f32n, nums};
use crate::tools::surface::{METRES, face_normal, split, y_up};

mod animation;
mod glb;
mod obj;
mod scene;
mod stl;
pub(crate) use animation::*;
pub use glb::*;
pub use obj::*;
pub use scene::*;
pub use stl::*;
