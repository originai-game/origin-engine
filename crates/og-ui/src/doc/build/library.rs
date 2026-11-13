//! What a UI has been given to build from: its documents, what defining
//! them told about every build, and the assets they name by path. Written
//! by the `define_*` calls ([`Ui::define_document`]), read by the builds.
use super::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct Library {
    /// By name.
    pub(crate) documents: BTreeMap<String, Arc<UiDocument>>,
    /// CSS `@keyframes` of every defined document, as `<doc>/<name>`.
    pub(crate) keyframes: BTreeMap<String, Arc<crate::motion::css::Keyframes>>,
    /// What structural selectors of every defined document read.
    pub(crate) structure: crate::doc::matching::Profile,
    /// Global rules can read facts in a different document from their subject.
    pub(crate) global_structure: crate::doc::matching::Profile,
    /// Some defined document writes a CSS-wide keyword on a property, so
    /// elements carry their winners to their children's `inherit`.
    pub(crate) wide_keywords: bool,
    /// Some defined document uses generated content, so builds keep the
    /// document-order counters and quote run.
    pub(crate) uses_run: bool,
    /// The textures documents name by asset path (a view transition's
    /// cover image), by path.
    pub(crate) textures: BTreeMap<String, og_draw::TextureHandle>,
    /// The alpha of the textures a `shape-outside: url()` names, by path
    /// ([`Ui::define_texture_alpha`]).
    pub(crate) alphas: BTreeMap<String, Arc<crate::layout::float::ImageAlpha>>,
    /// The Rive artboards elements can play, by id ([`Ui::define_rive`]).
    pub(crate) rive: BTreeMap<String, Arc<crate::rive::Artboard>>,
    /// The `type ui-material` assets documents name (`-og-material:
    /// "<path>" …`): each one's handle and its parameters' defaults.
    pub(crate) materials: BTreeMap<
        String,
        (
            og_draw::UiMaterialHandle,
            [[f32; 4]; og_draw::UI_MATERIAL_PARAMS],
        ),
    >,
}

impl Library {
    /// Takes a document, in place of one of the same name: its keyframes
    /// replace that one's, and what its rules need of every build is added
    /// to what the others need.
    pub(crate) fn define(&mut self, doc: UiDocument) {
        let declarations = || {
            doc.nodes
                .iter()
                .flat_map(|n| &n.rules)
                .flat_map(|r| &r.declarations)
        };
        self.uses_run |= declarations().any(|d| crate::doc::generated::longhand(&d.name));
        self.wide_keywords |= declarations().any(|d| {
            d.value.binding().is_some()
                || super::cascade::CARRIED_BY_DEFAULT.contains(&d.name.as_str())
                || !d.name.starts_with("--")
                    && matches!(
                        crate::doc::stylesheet::wide_keyword(&d.source),
                        Some("inherit" | "unset" | "revert" | "revert-layer")
                    )
        });
        let prefix = format!("{}/", doc.name);
        self.keyframes.retain(|name, _| !name.starts_with(&prefix));
        for (n, k) in &doc.sheet.keyframes {
            self.keyframes
                .insert(format!("{prefix}{n}"), Arc::new(k.clone()));
        }
        self.structure
            .merge(&crate::doc::matching::Profile::of_document(&doc));
        for selector in doc
            .nodes
            .iter()
            .flat_map(|n| &n.rules)
            .filter_map(|r| r.structure.as_ref())
            .filter(|s| s.global)
        {
            self.global_structure.add(selector);
        }
        self.documents.insert(doc.name.clone(), Arc::new(doc));
    }
}
