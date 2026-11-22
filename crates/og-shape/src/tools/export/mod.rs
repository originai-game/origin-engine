//! GLB, OBJ and STL from a [`Scene`] of named solids.

use std::collections::BTreeMap;
use std::fmt;

use crate::math::{DVec3, Frame, trig};
use crate::mesh::Solid;

use crate::tools::image::Texture;
use crate::tools::surface::{METRES, y_up};

pub mod scene;
pub use scene::*;
