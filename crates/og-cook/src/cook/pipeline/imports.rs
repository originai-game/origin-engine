//! Native import controls and captured inputs. Internal producers state only
//! their own controls; none constructs or parses an import author document.
use crate::cook::source_schemas::world::import::{self, ImportInput};
use og_core::hash::StableHasher;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Default)]
pub(crate) struct Settings {
    pub input: Option<Arc<ImportInput>>,
    /// Checked declaration identity, before the external bytes replace it.
    pub stamp: Vec<u8>,
    /// Explicit owning-module paths and importer-discovered dependencies.
    pub files: BTreeMap<String, Vec<u8>>,
    /// The conformance pass and sheet producers may require pixel encoding.
    pub pixel: bool,
    /// A generated sheet may already satisfy its renderer's art contract.
    pub conform_off: bool,
    /// An internally produced effect mesh has no prop face budget.
    pub effect_mesh: bool,
    /// District's internal native stamps, never an author settings document.
    pub carves: og_core::ogt::Block,
    /// Imported animation libraries share the same captured model boundary.
    pub libraries: Arc<std::sync::OnceLock<BTreeMap<String, crate::cook::anim::Library>>>,
}

impl Settings {
    /// Capture declared raw inputs before any source enters import/planning.
    pub fn new(
        input: ImportInput,
        stamp: Vec<u8>,
        read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
    ) -> Result<Self, String> {
        let mut settings = Self {
            input: Some(Arc::new(input)),
            stamp,
            ..Self::default()
        };
        let mut paths = vec![settings.source().expect("import input").to_string()];
        match settings.input.as_deref().expect("import input") {
            ImportInput::Model(d) => {
                if let Some(project) = d.process.as_ref().and_then(|p| p.project.as_ref()) {
                    paths.extend(project.views.iter().map(|v| v.image.clone()));
                }
                if let Some(f) = &d.fracture {
                    for m in [&f.interior, &f.mortar].into_iter().flatten() {
                        paths.extend(
                            [&m.map, &m.normal_map, &m.metalness_map, &m.ao_map]
                                .into_iter()
                                .flatten()
                                .cloned(),
                        );
                    }
                }
            }
            ImportInput::Texture(d) => {
                if let Some(import::Normal::Image { source }) = &d.normal {
                    paths.push(source.clone());
                }
                paths.extend(d.sway.iter().filter_map(|s| s.parts.clone()));
            }
            ImportInput::Rive(d) => {
                paths.extend(d.fonts.values().filter_map(|f| f.source.clone()));
            }
            _ => {}
        }
        paths.sort();
        paths.dedup();
        for path in paths {
            settings.files.insert(path.clone(), read(&path)?);
        }
        Ok(settings)
    }

    pub fn model(&self) -> Option<&import::Model> {
        match self.input.as_deref()? {
            ImportInput::Model(d) => Some(d),
            _ => None,
        }
    }

    pub fn texture(&self) -> Option<&import::Texture> {
        match self.input.as_deref()? {
            ImportInput::Texture(d) => Some(d),
            _ => None,
        }
    }

    pub fn sound(&self) -> Option<&import::Sound> {
        match self.input.as_deref()? {
            ImportInput::Sound(d) => Some(d),
            _ => None,
        }
    }

    pub fn font(&self) -> Option<&import::Font> {
        match self.input.as_deref()? {
            ImportInput::Font(d) => Some(d),
            _ => None,
        }
    }

    pub fn tilemap(&self) -> Option<&import::Tilemap> {
        match self.input.as_deref()? {
            ImportInput::Tilemap(d) => Some(d),
            _ => None,
        }
    }

    pub fn rive(&self) -> Option<&import::rive::Rive> {
        match self.input.as_deref()? {
            ImportInput::Rive(d) => Some(d),
            _ => None,
        }
    }

    pub fn source(&self) -> Option<&str> {
        Some(match self.input.as_deref()? {
            ImportInput::Model(d) => &d.source,
            ImportInput::Texture(d) => &d.source,
            ImportInput::Sound(d) => &d.source,
            ImportInput::Font(d) => &d.source,
            ImportInput::Tilemap(d) => &d.source,
            ImportInput::Rive(d) => &d.source,
        })
    }

    pub fn kind(&self) -> crate::cook::Kind {
        use crate::cook::{audio, font, model, riv, texture, tilemap};
        match self.input.as_deref().expect("import input") {
            ImportInput::Model(_) => &model::MODEL,
            ImportInput::Texture(_) => &texture::TEXTURE,
            ImportInput::Sound(_) => &audio::SOUND,
            ImportInput::Font(_) => &font::FONT,
            ImportInput::Tilemap(_) => &tilemap::TILEMAP,
            ImportInput::Rive(_) => &riv::RIVE,
        }
    }

    pub fn read(&self, file: &str) -> Result<Vec<u8>, String> {
        self.files
            .get(file)
            .cloned()
            .ok_or_else(|| format!("{file}: not a captured import input"))
    }

    pub fn hash(&self) -> u64 {
        let mut h = StableHasher::new();
        h.write(&self.stamp);
        h.write_u64(u64::from(self.pixel));
        h.write_u64(u64::from(self.conform_off));
        h.write_u64(u64::from(self.effect_mesh));
        for (path, bytes) in &self.files {
            h.write_str(path);
            h.write(bytes);
        }
        // District already owns the stamp's native entries and ordering. This
        // is an internal cache identity, not an author text or conversion path.
        let mut doc = og_core::ogt::Document::new("district-stamps", 1);
        doc.body = self.carves.clone();
        h.write_str(&doc.to_text().expect("native district stamps"));
        h.finish()
    }
}

impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Settings")
            .field("hash", &self.hash())
            .finish()
    }
}

/// Imports must exist before authored models, terrain and navigation capture
/// their operands. Keep the checked content for the final reference graph.
pub(crate) fn prepare(
    root: &std::path::Path,
    project: &crate::cook::source_project::Project,
    sources: &mut Vec<crate::cook::Source>,
    prepared: &mut BTreeMap<String, crate::cook::source_content::Content>,
) -> Result<(), crate::cook::CookError> {
    use crate::cook::{CookError, source_content, source_index, source_schemas, tilemap};
    let index = source_index::Index::open(root, project).map_err(|e| CookError(e.to_string()))?;
    let mut evaluator = source_content::Evaluator::default();
    for entry in index.entries.values().filter(|e| e.factory == "import") {
        let content = evaluator
            .evaluate(root, project, entry, &BTreeMap::new(), None)
            .map_err(|e| CookError(e.to_string()))?;
        let file: source_schemas::File = serde_json::from_value(content.data().root().into_json())
            .map_err(|e| CookError(e.to_string()))?;
        let input: ImportInput =
            serde_json::from_value(file.value).map_err(|e| CookError(e.to_string()))?;
        let module = root.join(&project.modules[&entry.module].root);
        let settings = Settings::new(input, content.data().as_bytes().to_vec(), &|path| {
            if !path.starts_with("assets/") && !path.starts_with("sources/") {
                return Err(format!(
                    "{path}: import inputs belong under the owning module's assets/ or sources/"
                ));
            }
            let checked = crate::cook::source_path::checked(&module, path)
                .map_err(|e| format!("{path}: {e}"))?;
            std::fs::read(checked).map_err(|e| format!("{path}: {e}"))
        })
        .map_err(|e| {
            CookError(source_content::diagnostic(content.source(), &entry.file, "", e).to_string())
        })?;
        let kind = settings.kind();
        let input_path = settings.source().expect("import input");
        let extension = crate::cook::ext(input_path);
        if !kind.extensions.contains(&extension.as_str()) {
            return Err(CookError(
                source_content::diagnostic(
                    content.source(),
                    &entry.file,
                    "/source",
                    format!("{input_path}: not a supported {} input", kind.name),
                )
                .to_string(),
            ));
        }
        let bytes = settings.read(input_path).map_err(CookError)?;
        let mut source = super::author::declaration(root, entry, kind, bytes);
        source.path = module.join(input_path);
        source.settings = settings;
        if kind == &tilemap::TILEMAP {
            let options =
                tilemap::MapSettings::from_input(source.settings.tilemap()).map_err(CookError)?;
            source.tilemap = Some(
                tilemap::import(
                    source.settings.source().expect("import input"),
                    &source.bytes,
                    &options,
                    &|id| index.entries.contains_key(id),
                )
                .map_err(CookError)?,
            );
        }
        sources.push(source);
        prepared.insert(entry.id.clone(), content);
    }
    Ok(())
}

/// Subresources the native import actually produces, not guessed filenames.
pub(crate) fn outputs(s: &crate::cook::Source) -> Result<BTreeMap<String, String>, String> {
    use crate::cook::{anim, anim_settings, audio, model, riv, texture};
    let mut out = BTreeMap::new();
    if s.kind == &model::MODEL {
        if s.gltf.as_ref().is_some_and(anim::wanted) || !anim_settings(s)?.clips.is_empty() {
            out.insert("anim".into(), "animation".into());
        }
        if s.fracture.is_some() {
            out.insert("fracture".into(), "fracture".into());
        }
        if s.settings.model().and_then(|d| d.options.impostor) == Some(true) {
            for suffix in ["impostor", "impostor-normal", "impostor-material"] {
                out.insert(suffix.into(), "texture".into());
            }
        }
    } else if s.kind == &texture::TEXTURE {
        if let Some(d) = s.settings.texture() {
            for (present, suffix) in [(d.normal.is_some(), "normal"), (d.sway.is_some(), "sway")] {
                if present {
                    out.insert(suffix.into(), "texture".into());
                }
            }
        }
    } else if s.kind == &audio::SOUND {
        let (rate, channels, frames) =
            audio::shape(s.settings.source().expect("import input"), &s.bytes)?;
        let settings = audio::SoundSettings::from_input(s.settings.sound())?;
        out.extend(
            settings
                .chunking(rate, channels, frames)
                .into_iter()
                .flat_map(|(_, count)| 1..count)
                .map(|n| (format!("chunk{n}"), "sound-chunk".into())),
        );
    } else if s.kind == &riv::RIVE {
        let (names, images, fonts) = riv::plan(s, &Default::default())?;
        out.extend(names.into_iter().map(|n| (n, "component".into())));
        out.extend(
            images
                .into_iter()
                .map(|n| (format!("image{n}"), "texture".into())),
        );
        out.extend(
            fonts
                .into_iter()
                .map(|f| (format!("font{}", f.index), "font".into())),
        );
    }
    Ok(out)
}
