//! Checked model declarations enter the existing glTF writer directly.
use super::*;
use crate::cook::formats::geometry;
use crate::cook::source_schemas::world::{geometry::Mirror as InputMirror, model as input};

pub(crate) fn build(
    input: &input::GeometryModel,
    detail: f64,
    models: &geometry::Models<'_>,
) -> Result<Built, String> {
    let mut materials = Vec::new();
    let mut images = Vec::new();
    let mut material_index = BTreeMap::new();
    let mut nodes = Vec::new();
    let mut children: Vec<Vec<usize>> = Vec::new();
    let mut pivots = BTreeMap::new();
    let mut roots = Vec::new();
    let mut bin = Bin::default();
    let mut meshes = Vec::new();
    let mut written = BTreeMap::new();
    let mut drawn = 0;
    let mut planted = false;
    let add_node = |n: serde_json::Value,
                    parent: Option<usize>,
                    nodes: &mut Vec<serde_json::Value>,
                    children: &mut Vec<Vec<usize>>,
                    roots: &mut Vec<usize>| {
        nodes.push(n);
        children.push(Vec::new());
        let i = nodes.len() - 1;
        match parent {
            Some(p) => children[p].push(i),
            None => roots.push(i),
        }
        i
    };
    for item in &input.items {
        let name = &item.name;
        if name.is_empty() {
            return Err("a model item needs a name".into());
        }
        match &item.input {
            input::ItemInput::Material(m) => {
                if material_index.contains_key(name) {
                    return Err(format!("material '{name}' is defined twice"));
                }
                materials.push(super::surface::material(name, m, &mut images, 0)?);
                material_index.insert(name.clone(), materials.len() - 1);
            }
            input::ItemInput::Pivot(p) => {
                if pivots.contains_key(name) {
                    return Err(format!("pivot '{name}' is defined twice"));
                }
                let parent =
                    p.parent
                        .as_ref()
                        .map(|p| {
                            pivots.get(p).copied().ok_or_else(|| {
                                format!("'{p}' is not a pivot defined above '{name}'")
                            })
                        })
                        .transpose()?;
                let p = geometry::placement(p.transform.as_ref())?;
                if p.scale.min_element() <= 0.0 {
                    return Err(format!(
                        "pivot '{name}' needs positive scale (mirror a part, not a pivot)"
                    ));
                }
                let i = add_node(
                    node(name, &p),
                    parent,
                    &mut nodes,
                    &mut children,
                    &mut roots,
                );
                pivots.insert(name.clone(), i);
            }
            input::ItemInput::Part(p) => {
                let material = p
                    .material
                    .as_ref()
                    .map(|m| {
                        material_index.get(m).copied().ok_or_else(|| {
                            format!("'{m}' is not a material defined above part '{name}'")
                        })
                    })
                    .transpose()?
                    .unwrap_or(usize::MAX);
                let pivot = p
                    .pivot
                    .as_ref()
                    .map(|p| {
                        pivots.get(p).copied().ok_or_else(|| {
                            format!("'{p}' is not a pivot defined above part '{name}'")
                        })
                    })
                    .transpose()?;
                let places = match &p.placements {
                    None => vec![(Placement::default(), Mirror::default())],
                    Some(places) => places
                        .iter()
                        .map(|p| {
                            let mirror = match p.mirror {
                                Some(InputMirror::X) => Mirror {
                                    x: true,
                                    ..Default::default()
                                },
                                Some(InputMirror::Y) => Mirror {
                                    y: true,
                                    ..Default::default()
                                },
                                Some(InputMirror::Z) => Mirror {
                                    z: true,
                                    ..Default::default()
                                },
                                _ => Mirror::default(),
                            };
                            Ok((geometry::placement(p.transform.as_ref())?, mirror))
                        })
                        .collect::<Result<Vec<_>, String>>()?,
                };
                if places.is_empty() {
                    return Err(format!("part '{name}' needs a placement"));
                }
                let mesh = geometry::build_at(&p.geometry, models, detail)
                    .map_err(|e| format!("part '{name}': {e}"))?;
                drawn += mesh.triangle_count() * places.len();
                for (k, (p, m)) in places.iter().enumerate() {
                    let mirrors = m.x || m.y || m.z || p.scale.min_element() < 0.0;
                    let (geo, local) = if mirrors {
                        let baked = og_shape::generators::merge(&[og_shape::generators::Part {
                            mesh: &mesh,
                            placement: *p,
                            mirror: *m,
                        }])?;
                        (std::borrow::Cow::Owned(baked), Placement::default())
                    } else {
                        if p.scale.min_element() == 0.0 {
                            return Err(format!("part '{name}' has a singular placement"));
                        }
                        (std::borrow::Cow::Borrowed(&mesh), *p)
                    };
                    let key = (geo.bits(), material);
                    let mi = *written.entry(key).or_insert_with(|| {
                        meshes.push(write_mesh(&mut bin, name, &geo, material));
                        meshes.len() - 1
                    });
                    let label = if places.len() == 1 {
                        name.clone()
                    } else {
                        format!("{name}[{k}]")
                    };
                    let mut n = node(&label, &local);
                    n["mesh"] = mi.into();
                    add_node(n, pivot, &mut nodes, &mut children, &mut roots);
                }
            }
            input::ItemInput::Plant(p) => {
                if planted {
                    return Err("a model grows one plant (its wind is the model's)".into());
                }
                planted = true;
                let (desc, look) = super::plant_input::plant(p, models)?;
                let grown = og_shape::generators::plant(&desc, detail)
                    .map_err(|e| format!("plant '{name}': {e}"))?;
                let mut named =
                    |m: Option<String>, own: serde_json::Value| -> Result<usize, String> {
                        match m {
                            Some(m) => material_index.get(&m).copied().ok_or_else(|| {
                                format!("'{m}' is not a material defined above plant '{name}'")
                            }),
                            None => {
                                materials.push(own);
                                Ok(materials.len() - 1)
                            }
                        }
                    };
                let wood = named(
                    look.wood,
                    serde_json::json!({
                        "name": format!("{name} bark"),
                        "pbrMetallicRoughness": {"baseColorFactor": [look.bark_color.x, look.bark_color.y, look.bark_color.z, 1.0], "metallicFactor": 0.0, "roughnessFactor": 0.9}
                    }),
                )?;
                let mut primitives = vec![write_primitive(
                    &mut bin,
                    &grown.wood,
                    wood,
                    Some((&grown.wood_wind, false)),
                )];
                drawn += grown.wood.triangle_count();
                let mut leaf = None;
                if !grown.leaves.indices.is_empty() {
                    let mut own = serde_json::json!({"name":format!("{name} leaves"), "pbrMetallicRoughness":{"baseColorFactor":[1.0,1.0,1.0,1.0],"metallicFactor":0.0,"roughnessFactor":0.6},"doubleSided":true});
                    if look.translucency > 0.0 {
                        own["extensions"] = serde_json::json!({ DIFFUSE_TRANSMISSION: {"diffuseTransmissionFactor":look.translucency} });
                    }
                    if let Some(extras) = foliage_extras(look.dome_normals, &look.season) {
                        own["extras"] = extras;
                    }
                    let material = named(look.leaf, own)?;
                    primitives.push(write_primitive(
                        &mut bin,
                        &grown.leaves,
                        material,
                        Some((&grown.leaf_wind, true)),
                    ));
                    drawn += grown.leaves.triangle_count();
                    leaf = Some(material);
                }
                meshes.push(serde_json::json!({"name":name,"primitives":primitives}));
                let mut n = node(name, &Placement::default());
                n["mesh"] = (meshes.len() - 1).into();
                let kites = og_shape::generators::plant_level(&desc, detail)
                    .map_err(|e| format!("plant '{name}': {e}"))?;
                if let Some(material) = leaf
                    && kites.leaves.triangle_count() < grown.leaves.triangle_count()
                {
                    let label = format!("{name} lod 1");
                    meshes.push(serde_json::json!({"name":label,"primitives":[
                        write_primitive(&mut bin, &grown.wood, wood, Some((&grown.wood_wind,false))),
                        write_primitive(&mut bin, &kites.leaves, material, Some((&kites.leaf_wind,true)))
                    ]}));
                    let mut l = node(&label, &Placement::default());
                    l["mesh"] = (meshes.len() - 1).into();
                    nodes.push(l);
                    children.push(Vec::new());
                    n["extensions"] = serde_json::json!({ MSFT_LOD: {"ids":[nodes.len()-1]} });
                }
                add_node(n, None, &mut nodes, &mut children, &mut roots);
            }
        }
    }
    if meshes.is_empty() {
        return Err("a model needs at least one part or a plant".into());
    }
    for m in &mut meshes {
        for p in m["primitives"].as_array_mut().into_iter().flatten() {
            if p["material"] == serde_json::json!(usize::MAX) {
                p.as_object_mut().expect("object").remove("material");
            }
        }
    }
    for (i, c) in children.iter().enumerate() {
        if !c.is_empty() {
            nodes[i]["children"] = serde_json::json!(c);
        }
    }
    let mut json = serde_json::json!({"asset":{"version":"2.0","generator":"og-cook file.model"},"scene":0,"scenes":[{"nodes":roots}],"nodes":nodes,"meshes":meshes});
    if !materials.is_empty() {
        json["materials"] = materials.into();
    }
    if !images.is_empty() {
        let (imgs, texs) = textures(&images);
        json["images"] = imgs;
        json["textures"] = texs;
        json["samplers"] = serde_json::json!([{"wrapS":10497,"wrapT":10497}]);
    }
    let used = extensions_used(&json);
    if !used.is_empty() {
        json["extensionsUsed"] = used.into();
    }
    Ok(Built { json, bin, drawn })
}
