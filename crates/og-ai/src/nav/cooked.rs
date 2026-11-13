//! The cooked navmesh (`og.NavMesh`): what og-cook bakes, decoded into
//! this module's heightfield, tiles, areas and links. The blob schema is
//! og-bundle's (`og_bundle::format`); the loaders live here because they
//! build og-ai's types. A blob's agent profiles are names and bytes, so
//! `Bundle::navmesh_profiles` and `Bundle::navmesh_profile` stay in
//! og-bundle.

use og_bundle::format;
use og_bundle::meshopt_codec;
use og_bundle::runtime::{Bundle, LoadError, invalid, one, table};
use og_core::glam::Vec3;

/// The navmesh loaders of a cooked bundle: bring it into scope to call
/// them on a [`Bundle`].
pub trait BundleNav {
    /// Decodes a navmesh blob (`og.NavMesh`): the heightfield and the tiles
    /// the cook built (link them with [`super::NavMesh::from_tiles`]).
    fn navmesh(
        &self,
        id: &str,
        bytes: &[u8],
    ) -> Result<(super::Heightfield, Vec<super::Tile>), LoadError>;

    /// A navmesh blob's baked areas (G2-4: the `areas` section, each a
    /// byte area id then its shape; empty in a blob without areas), which
    /// the mesh re-marks when an obstacle rebuilds a tile
    /// ([`super::NavMesh::from_tiles_with_areas`]).
    fn navmesh_areas(&self, id: &str, bytes: &[u8]) -> Result<Vec<(super::Shape, u8)>, LoadError>;

    /// A navmesh blob's baked off-mesh links (G2-4: the `links` section,
    /// each a `u32` id then the link; empty in a blob without links).
    fn navmesh_links(&self, id: &str, bytes: &[u8]) -> Result<Vec<(u32, super::Link)>, LoadError>;
}

impl BundleNav for Bundle {
    fn navmesh(
        &self,
        id: &str,
        bytes: &[u8],
    ) -> Result<(super::Heightfield, Vec<super::Tile>), LoadError> {
        let blob = self.blob::<format::NavMesh>(id, bytes)?;
        let m = one::<format::NavMeshMeta>(&blob)?;
        // The heightfield's two tables are meshopt vertex streams: `rows`
        // rows of u32s. A stream too short to hold its rows (256 of them
        // take a byte or more) is refused before anything is allocated.
        let stream = |name: &str, rows: usize, stride: usize| -> Result<Vec<u32>, LoadError> {
            let encoded = blob.raw(name)?;
            if rows == 0 {
                return Ok(Vec::new());
            }
            if rows > encoded.len().saturating_mul(256) {
                return Err(invalid(&blob, format!("{name} is cut short")));
            }
            let bytes = meshopt_codec::decode_vertex_buffer(encoded, rows, stride)
                .map_err(|e| invalid(&blob, format!("{name}: {e}")))?;
            Ok(bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|c| u32::from_le_bytes(*c))
                .collect())
        };
        let cells = (m.width as usize)
            .checked_mul(m.depth as usize)
            .ok_or_else(|| invalid(&blob, "the grid is too large"))?;
        // A column's spans follow the previous column's.
        let mut first = 0u32;
        let mut columns = Vec::with_capacity(cells);
        for count in stream("columns", cells, size_of::<format::NavColumnRow>())? {
            columns.push([first, count]);
            first = first
                .checked_add(count)
                .ok_or_else(|| invalid(&blob, "the columns hold too many spans"))?;
        }
        let spans = stream("spans", first as usize, size_of::<format::NavSpanRow>())?;
        let tiles = table::<format::NavTileRow>(&blob, "tiles")?;
        let verts = table::<format::NavVertRow>(&blob, "verts")?;
        let polys = table::<format::NavPolyRow>(&blob, "polys")?;
        let index_bytes = blob.raw("poly_verts")?;
        let indices: Vec<u32> = index_bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| u32::from_le_bytes(*c))
            .collect();
        let params = super::NavParams {
            cell_size: m.cell_size,
            cell_height: m.cell_height,
            agent_height: m.agent_height,
            agent_radius: m.agent_radius,
            agent_climb: m.agent_climb,
            max_slope: m.max_slope,
            tile_size: m.tile_size,
            max_error: m.max_error,
        };
        let field = super::Heightfield {
            params,
            origin: Vec3::new(m.origin_x, m.origin_y, m.origin_z),
            width: m.width,
            depth: m.depth,
            columns,
            spans: spans
                .as_chunks::<2>()
                .0
                .iter()
                .map(|&[floor, clear]| super::Span { floor, clear })
                .collect(),
        };
        // G2-4's polygon areas, one byte per row of `polys` (absent before:
        // every polygon is ground).
        let poly_areas = blob.raw_opt("poly_areas").unwrap_or_default();
        if !poly_areas.is_empty() && poly_areas.len() != polys.len() {
            return Err(invalid(
                &blob,
                "poly_areas does not hold one area per polygon",
            ));
        }
        let mut out = Vec::with_capacity(tiles.len());
        for t in &tiles {
            let vs = verts
                .get(t.vert_first as usize..(t.vert_first + t.vert_count) as usize)
                .ok_or_else(|| invalid(&blob, "a tile's vertices are out of range"))?;
            let ps = polys
                .get(t.poly_first as usize..(t.poly_first + t.poly_count) as usize)
                .ok_or_else(|| invalid(&blob, "a tile's polygons are out of range"))?;
            let mut tile = super::Tile {
                verts: vs.iter().map(|v| Vec3::new(v.x, v.y, v.z)).collect(),
                lattice: vs.iter().map(|v| (v.lx, v.lz)).collect(),
                polys: Vec::with_capacity(ps.len()),
            };
            for (k, p) in ps.iter().enumerate() {
                let area = poly_areas
                    .get(t.poly_first as usize + k)
                    .copied()
                    .unwrap_or(0);
                let idx = indices
                    .get(p.index_first as usize..(p.index_first + p.index_count) as usize)
                    .ok_or_else(|| invalid(&blob, "a polygon's vertices are out of range"))?;
                if idx.len() < 3 || idx.iter().any(|&i| i as usize >= tile.verts.len()) {
                    return Err(invalid(&blob, "a polygon names a vertex its tile lacks"));
                }
                tile.polys.push(super::Poly {
                    verts: idx.iter().map(|&i| i as u16).collect(),
                    links: vec![super::mesh::NONE; idx.len()],
                    region: p.region,
                    area,
                });
            }
            out.push(tile);
        }
        Ok((field, out))
    }

    fn navmesh_areas(&self, id: &str, bytes: &[u8]) -> Result<Vec<(super::Shape, u8)>, LoadError> {
        let blob = self.blob::<format::NavMesh>(id, bytes)?;
        let mut b = blob.raw_opt("areas").unwrap_or_default();
        let mut out = Vec::new();
        while let Some((&area, rest)) = b.split_first() {
            b = rest;
            let shape = super::Shape::read(&mut b)
                .ok_or_else(|| invalid(&blob, "an area's shape is cut short"))?;
            out.push((shape, area));
        }
        Ok(out)
    }

    fn navmesh_links(&self, id: &str, bytes: &[u8]) -> Result<Vec<(u32, super::Link)>, LoadError> {
        let blob = self.blob::<format::NavMesh>(id, bytes)?;
        let mut b = blob.raw_opt("links").unwrap_or_default();
        let mut out = Vec::new();
        while b.len() >= 4 {
            let link = u32::from_le_bytes(b[..4].try_into().unwrap());
            b = &b[4..];
            let l =
                super::Link::read(&mut b).ok_or_else(|| invalid(&blob, "a link is cut short"))?;
            out.push((link, l));
        }
        if !b.is_empty() {
            return Err(invalid(&blob, "the links section ends inside a link"));
        }
        Ok(out)
    }
}
