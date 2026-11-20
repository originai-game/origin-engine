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
    pub sheet: std::sync::Arc<Sheet>,
    /// Its rules over the default style's, read once for every document
    /// it dresses.
    pub(crate) global: std::sync::Arc<crate::doc::stylesheet::Global>,
    pub(crate) rules: std::sync::Arc<Vec<StyleRule>>,
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
        crate::doc::stylesheet::part_rules_from(source, css, files).map(|_| ())
    }

    /// `files` contains CSS source keyed by asset ID, including imports.
    /// The cook stores these sources verbatim; runtime uses the same parser.
    pub fn parse(directory: &str, files: Vec<(String, String)>) -> Result<Self, String> {
        let directory = directory.trim_matches('/');
        let css = Self::sheet(directory, &files)?;
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
        let source = format!("{directory}/");
        // Its sheets are read once, and laid over the default style as
        // the process read it.
        let ((rules, sheet), global) = crate::doc::stylesheet::pack_sheets(&source, &css, files)?;
        Ok(Self {
            name: directory.rsplit('/').next().unwrap_or(directory).to_owned(),
            global: std::sync::Arc::new(global),
            sheet,
            rules,
            assets: assets.into_iter().collect(),
        })
    }

    /// The pack's own sheet: its direct child CSS files `@import`ed in
    /// lexical order into `og-pack`. The default style's rules follow it.
    pub(crate) fn sheet(directory: &str, files: &[(String, String)]) -> Result<String, String> {
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
        let mut css = String::from("@layer og-default, og-components, og-pack, og-game;\n");
        for id in entries {
            use cssparser::ToCss;
            css.push_str(&format!(
                "@import {} layer(og-pack);\n",
                cssparser::Token::QuotedString(format!("/{id}").into()).to_css_string()
            ));
        }
        Ok(css)
    }

    /// Parses a document against this pack: [`crate::UiDocument::parse_with_imports`]
    /// with the pack's rules for its elements placed with its own. A
    /// document is parsed once, against the pack of the UI it is defined
    /// in ([`crate::Ui::parse_document`] reads that pack).
    pub fn parse_document(
        &self,
        source: &str,
        text: &str,
        imports: Vec<(String, String)>,
    ) -> Result<crate::UiDocument, Vec<String>> {
        crate::doc::svelte::parse_with_pack(source, text, imports, Some(self))
    }

    /// Whether `other` is this reading of the pack: a clone of it, not
    /// another parse of the same files.
    fn same(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.global, &other.global)
    }
}

impl crate::Ui {
    /// Bind cooked sound asset IDs used by the active CSS pack.
    pub fn set_css_sounds(&mut self, sounds: std::collections::BTreeMap<String, u64>) {
        *self.style.css_sounds_mut() = sounds;
    }

    /// Seed the checkpointed presentation random stream from the engine seed.
    pub fn seed_feedback(&mut self, seed: u64) {
        self.interaction.state_mut().feedback_rng = seed ^ 0x9e37_79b9_7f4a_7c15;
    }

    /// Sets the style pack: its tokens, its `::part()` rules, its fonts
    /// and its name, in place of the pack before it.
    ///
    /// A document carries the rules of the pack it was parsed against
    /// ([`Ui::parse_document`]) and is never read again, so the pack is
    /// set before the UI's documents are parsed and defined. A host that
    /// replaces the pack under loaded documents (a style edited under
    /// `og dev`) parses and defines them again; [`Ui::styled_by`] tells it
    /// when.
    pub fn set_style_pack(&mut self, pack: &StylePack) -> Result<(), String> {
        self.docs.library.pack = Some(std::sync::Arc::new(pack.clone()));
        self.style
            .set_part_sheet((pack.rules.clone(), pack.sheet.clone()));
        self.style.root_inputs_changed();
        self.set_style_name(&pack.name);
        // A pack's faces back the roles it gives no font (its `--font-*`).
        self.back_role_fonts();
        self.env.fonts.link();
        self.layout.texts.clear();
        self.read_theme_tokens();
        self.tree.invalidation_mut().epoch += 1;
        Ok(())
    }

    /// Forgets the style pack: a document parsed from now on
    /// ([`Ui::parse_document`]) takes the default style's rules. The part
    /// stylesheet is the caller's to replace ([`Ui::set_part_css`]).
    pub fn clear_style_pack(&mut self) {
        self.docs.library.pack = None;
    }

    /// Whether `pack` is the one in effect (none: the default style), so
    /// the UI's documents were parsed against it.
    pub fn styled_by(&self, pack: Option<&StylePack>) -> bool {
        match (&self.docs.library.pack, pack) {
            (None, None) => true,
            (Some(mine), Some(pack)) => mine.same(pack),
            _ => false,
        }
    }

    /// The documents defined here that read otherwise under the style in
    /// effect than under the one they were parsed against: a host that
    /// replaced the pack or the default style under them (a style edited
    /// under `og dev`) parses and defines these again, and only these. A
    /// changed selector, layer, keyframes or at-rule names them all; a
    /// changed token (a `:root` custom property) names none, since the
    /// root cascade carries it.
    pub fn restyled_documents(&self) -> Vec<String> {
        let now = crate::doc::stylesheet::global_of(self.docs.library.pack.as_deref());
        let mut changes = Default::default();
        self.docs
            .library
            .documents
            .iter()
            .filter(|(_, d)| !d.styled.holds(&now, &mut changes))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Parses a document against the style pack in effect, or the default
    /// style without one: the one reading of its text. Give the
    /// stylesheets its `@import`s name as for
    /// [`crate::UiDocument::parse_with_imports`].
    pub fn parse_document(
        &self,
        source: &str,
        text: &str,
        imports: Vec<(String, String)>,
    ) -> Result<crate::UiDocument, Vec<String>> {
        crate::doc::svelte::parse_with_pack(
            source,
            text,
            imports,
            self.docs.library.pack.as_deref(),
        )
    }
}
