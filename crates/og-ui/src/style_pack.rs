//! A style pack is CSS and assets in a directory, with no manifest.
//! Direct child CSS files apply in lexical order in `og-pack`; nested
//! sheets are opt-in through standard `@import` rules.

use crate::doc::{
    StyleRule,
    at_rules::{self, Sheet},
};

#[derive(Clone, Debug)]
pub struct StylePack {
    pub name: String,
    pub sheet: Sheet,
    pub(crate) css: String,
    pub(crate) rules: Vec<StyleRule>,
    /// Resolved asset IDs used by CSS url(), excluding imported stylesheets.
    pub assets: Vec<String>,
}

impl StylePack {
    /// Check one author source with its own filename and line numbers.
    /// Imports are supplied by asset ID, as for `parse`.
    pub fn validate_source(
        source: &str,
        css: &str,
        files: Vec<(String, String)>,
    ) -> Result<(), String> {
        crate::doc::stylesheet::pack_extensions(source, css)?;
        crate::doc::stylesheet::part_rules_from(source, css, files, true).map(|_| ())
    }

    /// `files` contains CSS source keyed by asset ID, including imports.
    /// The cook stores these sources verbatim; runtime uses the same parser.
    pub fn parse(directory: &str, files: Vec<(String, String)>) -> Result<Self, String> {
        let directory = directory.trim_matches('/');
        let mut entries: Vec<_> = files
            .iter()
            .filter(|(id, _)| {
                id.rsplit_once('/')
                    .is_some_and(|(parent, _)| parent == directory)
            })
            .map(|(id, _)| id.as_str())
            .collect();
        entries.sort_unstable();
        if entries.is_empty() {
            return Err(format!(
                "{directory}: a style pack needs at least one .css file directly in its directory"
            ));
        }
        let mut css = String::from("@layer og-default, og-kit, og-pack, og-game;\n");
        for id in entries {
            use cssparser::ToCss;
            css.push_str(&format!(
                "@import {} layer(og-pack);\n",
                cssparser::Token::QuotedString(format!("/{id}").into()).to_css_string()
            ));
        }
        css.push_str(crate::DEFAULT_CSS);
        let mut assets = std::collections::BTreeSet::new();
        for (source, text) in &files {
            crate::doc::stylesheet::pack_extensions(source, text)?;
            for url in at_rules::urls(text) {
                let id = at_rules::resolve(source, &url);
                if !files.iter().any(|(path, _)| *path == id) {
                    assets.insert(id);
                }
            }
        }
        let (rules, sheet) =
            crate::doc::stylesheet::part_rules_from(&format!("{directory}/"), &css, files, true)?;
        Ok(Self {
            name: directory.rsplit('/').next().unwrap_or(directory).to_owned(),
            css,
            sheet,
            rules,
            assets: assets.into_iter().collect(),
        })
    }

    /// Reparse a document against this pack, preserving its authored source.
    pub fn apply_document(
        &self,
        doc: &crate::UiDocument,
    ) -> Result<crate::UiDocument, Vec<String>> {
        crate::doc::svelte::parse_with_pack(
            &doc.source,
            &doc.text,
            doc.sheet.imports.clone(),
            Some(self),
        )
    }
}

impl crate::Ui {
    /// Bind cooked sound asset IDs used by the active CSS pack.
    pub fn set_css_sounds(&mut self, sounds: std::collections::BTreeMap<String, u64>) {
        self.css_sounds = sounds;
    }

    /// Seed the checkpointed presentation random stream from the engine seed.
    pub fn seed_feedback(&mut self, seed: u64) {
        self.state.feedback_rng = seed ^ 0x9e37_79b9_7f4a_7c15;
    }

    /// Replace the active pack without retaining a previous pack's rules.
    /// Game document styles remain in their own scopes and higher layers.
    pub fn set_style_pack(&mut self, pack: &StylePack) -> Result<(), String> {
        self.style_documents(Some(pack))?;
        self.part_rules.clone_from(&pack.rules);
        self.part_sheet.clone_from(&pack.sheet);
        self.set_style_name(&pack.name);
        self.read_theme_tokens();
        self.kept.epoch += 1;
        Ok(())
    }

    /// Rebuild authored document rules when a pack changes or is cleared.
    pub fn style_documents(&mut self, pack: Option<&StylePack>) -> Result<(), String> {
        let docs = self
            .documents()
            .iter()
            .filter_map(|name| self.document(name))
            .map(|doc| {
                crate::doc::svelte::parse_with_pack(
                    &doc.source,
                    &doc.text,
                    doc.sheet.imports.clone(),
                    pack,
                )
                .map_err(|e| e.join("\n"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        for doc in docs {
            self.define_document(doc);
        }
        Ok(())
    }
}
