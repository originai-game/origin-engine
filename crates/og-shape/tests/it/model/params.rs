//! Program parameters: `"$name"` takes a variant's value over the
//! program's own, and a name either side does not declare is an error.

use og_shape::model::program::{Params, run, with_params};

const PROGRAM: &str = r#"{"name":"crate","params":{"width":400,"height":300},
  "ops":[{"op":"box","name":"body","width":"$width","depth":250,"height":"$height"}]}"#;

fn size(program: &str) -> [f64; 3] {
    let m = run(program).unwrap();
    let built = m.build().unwrap();
    let s = built.scene();
    let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
    for p in &s.parts {
        for q in &p.solid.positions {
            for (k, v) in [q.x, q.y, q.z].into_iter().enumerate() {
                lo[k] = lo[k].min(v);
                hi[k] = hi[k].max(v);
            }
        }
    }
    [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]]
}

#[test]
fn a_variant_sets_what_it_names_and_the_program_the_rest() {
    // Replayed as it is: the program's own values.
    assert_eq!(size(PROGRAM), [400.0, 250.0, 300.0]);
    let tall = with_params(PROGRAM, &Params::from([("height".into(), 1100.0)])).unwrap();
    assert_eq!(size(&tall), [400.0, 250.0, 1100.0]);
    // The same values give the same text, so the same bytes.
    assert_eq!(
        tall,
        with_params(PROGRAM, &Params::from([("height".into(), 1100.0)])).unwrap()
    );
    let unknown = with_params(PROGRAM, &Params::from([("depth".into(), 1.0)]));
    assert!(format!("{:?}", unknown.unwrap_err()).contains("depth"));
    let undeclared = PROGRAM.replace("\"$height\"", "\"$tall\"");
    assert!(
        format!(
            "{:?}",
            with_params(&undeclared, &Params::new()).unwrap_err()
        )
        .contains("tall")
    );
    // Expressions: precedence, parentheses, unary minus, and errors that
    // name the expression.
    let derived = PROGRAM.replace(
        "\"depth\":250",
        "\"depth\":\"($width - 100) / 2 * -1 + 2 * $height\"",
    );
    assert_eq!(size(&derived), [400.0, 450.0, 300.0]);
    for wrong in ["$width / 0", "($width", "$width $height", "$width ^ 2"] {
        let p = PROGRAM.replace("\"depth\":250", &format!("\"depth\":\"{wrong}\""));
        let e = format!("{:?}", with_params(&p, &Params::new()).unwrap_err());
        assert!(
            e.contains(&wrong.replace('$', "")) || e.contains(wrong),
            "{e}"
        );
    }
    let bad = PROGRAM.replace("\"height\":300", "\"height\":\"x\"");
    assert!(with_params(&bad, &Params::new()).is_err());
}
