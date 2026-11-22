use super::*;

/// A Wavefront file's triangles, each three positions.
fn obj_triangles(text: &str) -> Vec<[[f64; 3]; 3]> {
    let mut points = Vec::new();
    let mut out = Vec::new();
    for line in text.lines() {
        let mut words = line.split_whitespace();
        match words.next() {
            Some("v") => {
                let p: Vec<f64> = words.map(|w| w.parse().unwrap()).collect();
                points.push([p[0], p[1], p[2]]);
            }
            Some("f") => {
                let ids: Vec<usize> = words
                    .map(|w| w.split('/').next().unwrap().parse::<usize>().unwrap() - 1)
                    .collect();
                for k in 1..ids.len() - 1 {
                    out.push([points[ids[0]], points[ids[k]], points[ids[k + 1]]]);
                }
            }
            _ => {}
        }
    }
    out
}

/// Draco's own test files (`third_party/draco/testdata`), at
/// compression levels that between them use sequential and
/// EdgeBreaker connectivity, both traversals, and the difference,
/// parallelogram, constrained multi-parallelogram and texture
/// coordinate predictions: each decodes to the triangles of the
/// Wavefront file it was made from, to its quantization.
#[test]
fn draco_s_test_files_decode_to_their_sources() {
    macro_rules! file {
        ($name:literal) => {
            include_bytes!(concat!("../../../../../third_party/draco/testdata/", $name)).as_slice()
        };
    }
    let cases: [(&[u8], &[u8], usize); 6] = [
        (
            file!("test_nm.obj.edgebreaker.cl10.2.2.drc"),
            file!("test_nm.obj"),
            0,
        ),
        (
            file!("test_nm.obj.edgebreaker.cl4.2.2.drc"),
            file!("test_nm.obj"),
            0,
        ),
        (
            file!("test_nm.obj.sequential.cl3.2.2.drc"),
            file!("test_nm.obj"),
            0,
        ),
        (
            file!("cube_att.obj.edgebreaker.cl10.2.2.drc"),
            file!("cube_att.obj"),
            24,
        ),
        (
            file!("cube_att.obj.edgebreaker.cl4.2.2.drc"),
            file!("cube_att.obj"),
            24,
        ),
        (
            file!("cube_att.obj.sequential.cl3.2.2.drc"),
            file!("cube_att.obj"),
            24,
        ),
    ];
    for (k, (packed, source, uvs)) in cases.into_iter().enumerate() {
        let d = decode(packed).unwrap_or_else(|e| panic!("case {k}: {e}"));
        let mut want = obj_triangles(std::str::from_utf8(source).unwrap());
        assert_eq!(d.faces.len(), want.len(), "case {k}");
        let position = d
            .attributes
            .iter()
            .find(|a| a.components == 3 && !a.values.is_empty())
            .unwrap();
        assert_eq!(position.values.len(), d.points * 3);
        for f in &d.faces {
            let t = f.map(|v| {
                let p = &position.values[v as usize * 3..v as usize * 3 + 3];
                [p[0], p[1], p[2]]
            });
            // The same triangle, from any of its corners.
            let found = want.iter().position(|w| {
                (0..3).any(|turn| {
                    (0..3).all(|c| (0..3).all(|x| (t[c][x] - w[(c + turn) % 3][x]).abs() < 2e-3))
                })
            });
            want.swap_remove(
                found.unwrap_or_else(|| panic!("case {k}: {t:?} is not in the source")),
            );
        }
        // The cube's texture coordinates: a pair a point, in 0..=1.
        let uv = d.attributes.iter().find(|a| a.components == 2);
        assert_eq!(uv.map_or(0, |a| a.values.len()), uvs * 2, "case {k}");
        assert!(uv.is_none_or(|a| a.values.iter().all(|v| (-1e-3..=1.001).contains(v))));
    }
    // Damaged, a file is an error or some mesh, and not a panic.
    for (packed, _, _) in cases {
        for at in 0..packed.len() {
            for flip in [0xff, 0x55] {
                let mut broken = packed.to_vec();
                broken[at] ^= flip;
                let _ = decode(&broken);
            }
        }
    }
    // Cut short, a file is an error and not a panic.
    let whole = file!("test_nm.obj.edgebreaker.cl10.2.2.drc");
    for cut in [0, 5, 11, 40, whole.len() / 2, whole.len() - 1] {
        assert!(decode(&whole[..cut]).is_err(), "cut at {cut}");
    }
}
