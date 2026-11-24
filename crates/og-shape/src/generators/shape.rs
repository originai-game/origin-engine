//! three.js `ShapeGeometry` and `ShapeUtils`: a flat shape with holes,
//! triangulated by [`crate::generators::earcut`]. A shape arrives as points (the
//! TypeScript `og.Shape` flattens its curves with three.js's
//! `extractPoints(curveSegments)` before it crosses the wire), so the
//! counts and the vertex order are three.js's for the same points.

use crate::generators::{Mesh, earcut::earcut, finish};
use glam::{DVec2, DVec3};

/// A shape's outline and its holes, as three.js `Shape.extractPoints`
/// returns them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapePoints {
    pub points: Vec<DVec2>,
    pub holes: Vec<Vec<DVec2>>,
}

/// three.js `ShapeUtils.area`: positive counter-clockwise (y up).
pub fn area(contour: &[DVec2]) -> f64 {
    let n = contour.len();
    let mut a = 0.0;
    let mut p = n.wrapping_sub(1);
    for q in 0..n {
        a += contour[p].x * contour[q].y - contour[q].x * contour[p].y;
        p = q;
    }
    a * 0.5
}

/// three.js `ShapeUtils.isClockWise`.
pub fn is_clockwise(points: &[DVec2]) -> bool {
    area(points) < 0.0
}

/// three.js's `removeDupEndPts`: a closed outline's repeated last point.
fn remove_dup_end(points: &mut Vec<DVec2>) {
    let l = points.len();
    if l > 2 && points[l - 1] == points[0] {
        points.pop();
    }
}

/// three.js `ShapeUtils.triangulateShape`: drops each ring's repeated end
/// point (in place, as three.js does, so the caller's vertex list loses it
/// too) and triangulates with earcut; the indices count the contour's
/// points, then each hole's.
pub fn triangulate_shape(contour: &mut Vec<DVec2>, holes: &mut [Vec<DVec2>]) -> Vec<[usize; 3]> {
    remove_dup_end(contour);
    let mut data: Vec<f64> = contour.iter().flat_map(|p| [p.x, p.y]).collect();
    let mut hole_indices = Vec::new();
    let mut hole_index = contour.len();
    for h in holes.iter_mut() {
        remove_dup_end(h);
    }
    for h in holes.iter() {
        hole_indices.push(hole_index);
        hole_index += h.len();
        data.extend(h.iter().flat_map(|p| [p.x, p.y]));
    }
    earcut(&data, &hole_indices)
        .as_chunks::<3>()
        .0
        .iter()
        .map(|c| [c[0], c[1], c[2]])
        .collect()
}

/// three.js `ShapeGeometry(shapes)`: each shape in the xy plane facing +Z,
/// its outline made clockwise and its holes counter-clockwise first,
/// world uvs (x, y; v flipped to run down the image).
pub fn shape_geometry(shapes: &[ShapePoints]) -> Mesh {
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    for shape in shapes {
        let offset = positions.len() as u32;
        let mut verts = shape.points.clone();
        let mut holes = shape.holes.clone();
        if !is_clockwise(&verts) {
            verts.reverse();
        }
        for h in holes.iter_mut() {
            if is_clockwise(h) {
                h.reverse();
            }
        }
        let faces = triangulate_shape(&mut verts, &mut holes);
        for h in &holes {
            verts.extend_from_slice(h);
        }
        for v in &verts {
            positions.push(DVec3::new(v.x, v.y, 0.0));
            uvs.push([v.x, 1.0 - v.y]);
        }
        for f in faces {
            indices.extend(f.map(|i| i as u32 + offset));
        }
    }
    let normals = vec![DVec3::Z; positions.len()];
    finish(&positions, &normals, &uvs, indices)
}
