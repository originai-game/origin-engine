//! Every problem a model's measures show, in one list, each with the
//! option that addresses it, so one pass fixes them all. A problem is a
//! fact against a rule the mesh format or the game's look states: an open
//! or non-manifold edge, a triangle facing against its neighbours, a
//! degenerate or repeated triangle, a floating piece, a budget the look
//! states and the model is past. Nothing here judges taste: a colour's
//! distance from the palette is a fact in `look`, never an issue.
//!
//! Entry: [`issues`], over [`totals`] of a measure report.

use serde_json::{Value, json};

/// The issues of a mesh's measures (`og_shape::tools::MeshMeasures` as
/// JSON) and its look facts (`look` of `og model measure`, if any). Each
/// is `{issue, count?, ratio?, next}`; `next` is the option or command
/// that addresses it. An empty list is a model with none of them.
pub fn issues(measures: &Value, look: Option<&Value>) -> Vec<Value> {
    let mut out = Vec::new();
    let count = |k: &str| measures.get(k).and_then(Value::as_u64).unwrap_or(0);
    for (key, issue, next) in [
        (
            "boundary_edges",
            "openEdges",
            "og model process --holes <rim edges> (or --weld <mm> where the cracks are seams)",
        ),
        (
            "non_manifold_edges",
            "nonManifoldEdges",
            "og model process --non-manifold",
        ),
        (
            "inconsistent_edges",
            "flippedTriangles",
            "og model repair (it orients every piece)",
        ),
        (
            "degenerate_triangles",
            "degenerateTriangles",
            "og model repair",
        ),
        (
            "duplicate_triangles",
            "duplicateTriangles",
            "og model repair",
        ),
        (
            "floaters",
            "floatingPieces",
            "og model process --floaters <share of the diagonal>",
        ),
    ] {
        let n = count(key);
        if n > 0 {
            out.push(json!({ "issue": issue, "count": n, "next": next }));
        }
    }
    let Some(look) = look else { return out };
    for (ratio, budget, issue, next) in [
        (
            "facesRatio",
            "propFaces",
            "overPropFaces",
            "og model process --budget <the look's propFaces>",
        ),
        (
            "characterFacesRatio",
            "characterFaces",
            "overCharacterFaces",
            "og model process --budget <the look's characterFaces>",
        ),
        (
            "coloursRatio",
            "maxColors",
            "overColours",
            "fewer materials or a texture with fewer colours",
        ),
    ] {
        if let Some(r) = look.get(ratio).and_then(Value::as_f64).filter(|r| *r > 1.0) {
            out.push(json!({
                "issue": issue,
                "ratio": r,
                "budget": look.get(budget),
                "next": next,
            }));
        }
    }
    out
}

/// A measure report's defect counts in the mesh measures' names, every
/// part's summed (a program's parts camelCase, a mesh file's under
/// `measures`): parts that only touch are an assembly, not a
/// non-manifold edge, so the joined `whole` is used only without parts.
pub fn totals(report: &Value) -> Value {
    let parts = report.get("parts").and_then(Value::as_array);
    if parts.is_none_or(|p| p.is_empty())
        && let Some(w) = report.get("whole").filter(|w| w.is_object())
    {
        return w.clone();
    }
    let keys = [
        ("boundary_edges", "boundaryEdges"),
        ("non_manifold_edges", "nonManifoldEdges"),
        ("inconsistent_edges", "inconsistentEdges"),
        ("degenerate_triangles", "degenerateTriangles"),
        ("duplicate_triangles", "duplicateTriangles"),
        ("floaters", "floaters"),
    ];
    let mut out = serde_json::Map::new();
    for p in report
        .get("parts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let m = p.get("measures").unwrap_or(p);
        for (snake, camel) in keys {
            let n = m
                .get(snake)
                .or_else(|| m.get(camel))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let e = out.entry(snake).or_insert(json!(0));
            *e = json!(e.as_u64().unwrap_or(0) + n);
        }
    }
    Value::Object(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every defect at once, and a budget overrun the look states; a
    /// budget it does not state (null) and palette distance are no issue.
    #[test]
    fn every_problem_is_listed_in_one_call() {
        let m = json!({ "boundary_edges": 37, "non_manifold_edges": 17, "floaters": 0, "inconsistent_edges": 2 });
        let look = json!({ "facesRatio": 1.5, "propFaces": 1000, "characterFacesRatio": null,
                           "palette": [{ "deltaE": 40.0 }] });
        let got: Vec<String> = issues(&m, Some(&look))
            .iter()
            .map(|i| i["issue"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(
            got,
            [
                "openEdges",
                "nonManifoldEdges",
                "flippedTriangles",
                "overPropFaces"
            ]
        );
        assert!(issues(&json!({ "boundary_edges": 0 }), None).is_empty());
    }
}
