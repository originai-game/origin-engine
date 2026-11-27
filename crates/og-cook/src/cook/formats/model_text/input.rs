//! Captured native model inputs, shared by import and face-budget rebuilding.
use super::*;
use crate::cook::source_schemas::world::{geometry::Geometry, model as input};
use std::sync::OnceLock;

pub(crate) struct Prepared {
    pub input: input::ModelInput,
    pub program: Option<super::super::model_program::Program>,
    pub operands: OnceLock<BTreeMap<(String, Option<String>), Mesh>>,
    pub images: BTreeMap<String, Vec<u8>>,
    pub hash: Vec<u8>,
    pub libraries: OnceLock<BTreeMap<String, super::super::anim::Library>>,
}

impl Prepared {
    pub fn new(
        input: input::ModelInput,
        hash: Vec<u8>,
        read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
        program: Option<super::super::model_program::Program>,
    ) -> Result<Self, String> {
        let mut paths = Vec::new();
        let mut surface = |m: &crate::cook::source_schemas::world::materials::Material| {
            paths.extend(
                [&m.map, &m.normal_map, &m.metalness_map, &m.ao_map]
                    .into_iter()
                    .flatten()
                    .cloned(),
            );
        };
        match &input {
            input::ModelInput::Geometry(d) => {
                for item in &d.items {
                    if let input::ItemInput::Material(m) = &item.input {
                        surface(m);
                    }
                }
                if let Some(f) = &d.fracture {
                    for m in [&f.interior, &f.mortar].into_iter().flatten() {
                        surface(m);
                    }
                }
            }
            input::ModelInput::Program(d) => {
                for m in &d.materials {
                    paths.extend(
                        [
                            &m.map,
                            &m.normal_map,
                            &m.roughness_map,
                            &m.emissive_map,
                            &m.ao_map,
                        ]
                        .into_iter()
                        .flatten()
                        .cloned(),
                    );
                }
            }
        }
        let processing = match &input {
            input::ModelInput::Geometry(d) => d.process.as_ref(),
            input::ModelInput::Program(d) => d.process.as_ref(),
        };
        if let Some(p) = processing.and_then(|p| p.project.as_ref()) {
            paths.extend(p.views.iter().map(|v| v.image.clone()));
        }
        paths.sort();
        paths.dedup();
        let mut images = BTreeMap::new();
        for path in paths {
            if !path.starts_with("assets/") && !path.starts_with("sources/") {
                return Err(format!(
                    "{path}: model inputs belong under the owning module's assets/ or sources/"
                ));
            }
            let bytes = read(&path)?;
            images.insert(path, bytes);
        }
        Ok(Self {
            input,
            program,
            operands: OnceLock::new(),
            images,
            hash,
            libraries: OnceLock::new(),
        })
    }

    pub fn process(&self) -> Option<&input::process::Process> {
        match &self.input {
            input::ModelInput::Geometry(d) => d.process.as_ref(),
            input::ModelInput::Program(d) => d.process.as_ref(),
        }
    }

    pub fn conform(&self) -> Option<&input::Conform> {
        match &self.input {
            input::ModelInput::Geometry(d) => d.conform.as_ref(),
            input::ModelInput::Program(d) => d.conform.as_ref(),
        }
    }

    pub fn options(&self) -> Option<&input::ModelOptions> {
        match &self.input {
            input::ModelInput::Geometry(d) => d.options.as_ref(),
            input::ModelInput::Program(d) => d.options.as_ref(),
        }
    }

    pub fn fracture(&self) -> Result<Option<crate::cook::fracture::Settings>, String> {
        match &self.input {
            input::ModelInput::Geometry(d) => d
                .fracture
                .as_ref()
                .map(crate::cook::fracture::Settings::from_input)
                .transpose(),
            _ => Ok(None),
        }
    }

    pub fn references(&self) -> Vec<(String, Option<String>)> {
        fn geometry(g: &Geometry, out: &mut Vec<(String, Option<String>)>) {
            match g {
                Geometry::ModelGeometry(d) => out.push((d.source.id.clone(), d.node.clone())),
                Geometry::CsgGeometry(d) => {
                    geometry(&d.a.geometry, out);
                    geometry(&d.b.geometry, out);
                }
                Geometry::MergeGeometries(d) => {
                    for p in &d.parts {
                        geometry(&p.geometry, out);
                    }
                }
                _ => {}
            }
        }
        let mut out = Vec::new();
        if let input::ModelInput::Geometry(d) = &self.input {
            for item in &d.items {
                match &item.input {
                    input::ItemInput::Part(p) => geometry(&p.geometry, &mut out),
                    input::ItemInput::Plant(p) => {
                        for g in [&p.leaf_geometry, &p.flower_geometry].into_iter().flatten() {
                            geometry(g, &mut out);
                        }
                    }
                    _ => {}
                }
            }
        }
        out.sort();
        out.dedup();
        out
    }

    pub fn read(&self, file: &str) -> Result<Vec<u8>, String> {
        self.images
            .get(file)
            .cloned()
            .ok_or_else(|| format!("{file}: not a captured model input"))
    }

    pub fn glb(&self, budget: u32) -> Result<(Vec<u8>, f64), String> {
        let input::ModelInput::Geometry(input) = &self.input else {
            return self
                .program
                .as_ref()
                .expect("checked program model")
                .glb(&|f| self.read(f))
                .map(|w| (w.glb, 1.0));
        };
        let models = |id: &str, node: Option<&str>| {
            self.operands
                .get()
                .and_then(|m| m.get(&(id.to_string(), node.map(str::to_string))))
                .cloned()
                .ok_or_else(|| format!("{id}: modelGeometry has no imported operand"))
        };
        let b = native::build(input, 1.0, &models)?;
        if budget == 0 || b.drawn <= budget as usize {
            return Ok((b.bin.finish(b.json)?, 1.0));
        }
        let (mut lo, mut hi) = (0u32, 256u32);
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            if native::build(input, f64::from(mid) / 256.0, &models)?.drawn <= budget as usize {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let detail = f64::from(lo) / 256.0;
        let b = native::build(input, detail, &models)?;
        Ok((b.bin.finish(b.json)?, detail))
    }
}
