<meta name="screen" content="creator-tools" data-layer="menu" data-focus="" data-under="none" data-input="all" data-pause="false" data-capture="true" data-remember="true" data-back="exit">
<meta name="strings" content="./creator-tools.en.ogt">
<meta name="prop:tool" content="string" data-doc="select, move, rotate or scale.">
<meta name="prop:playing" content="bool" data-doc="The world is playing.">
<meta name="prop:selected" content="string" data-doc="The selected object's name (empty: none).">
<meta name="prop:panel" content="string" data-doc="On a phone, the open panel: hierarchy, inspector or catalogue.">
<meta name="prop:dirty" content="bool" data-doc="Unsaved changes.">
<meta name="action:back" content="Exit">
<meta name="fixture:default" content="{&quot;dirty&quot;:true,&quot;panel&quot;:&quot;inspector&quot;,&quot;playing&quot;:false,&quot;selected&quot;:&quot;Lantern_03&quot;,&quot;tool&quot;:&quot;move&quot;}">

<column id="Editor">
  <row id="Toolbar">
    <button id="Select" variant="ghost" class="compact" class:selection-off={!(($tool == "select"))} class:selection-on={($tool == "select")}>{t("screen.creator-tools.select")}</button>
    <button id="Move" variant="ghost" class="compact" class:selection-off={!(($tool == "move"))} class:selection-on={($tool == "move")}>{t("screen.creator-tools.move")}</button>
    <button id="Rotate" variant="ghost" class="compact" class:selection-off={!(($tool == "rotate"))} class:selection-on={($tool == "rotate")}>{t("screen.creator-tools.rotate")}</button>
    <button id="Scale" variant="ghost" class="compact" class:selection-off={!(($tool == "scale"))} class:selection-on={($tool == "scale")}>{t("screen.creator-tools.scale")}</button>
    <spacer id="Node6"></spacer>
    <button id="Undo" class="compact" variant="quiet" onclick={undo}>{t("screen.creator-tools.undo")}</button>
    <button id="Redo" class="compact" variant="quiet" onclick={redo}>{t("screen.creator-tools.redo")}</button>
    <button id="Play" class="compact" variant="primary" onclick={play} class:run-stopped={!($playing)} class:run-playing={$playing}>{t("screen.creator-tools.play")}</button>
    <button id="Save" class="compact" disabled={!($dirty)} onclick={save}>{t("screen.common.save")}</button>
  </row>
  <grid id="Body">
    <column id="TreePanel" class:chosen={$panel == 'hierarchy'}>
      <text id="TreePanelTitle">{t("screen.creator-tools.hierarchy")}</text>
      <column id="TreePanelHierarchy">{@render treePanelHierarchy?.()}</column>
    </column>
    <column id="View">{@render view?.()}</column>
    <row id="Tabs" visible="false">
      <button id="TabsHierarchy" class="compact" variant="ghost">{t("screen.creator-tools.hierarchy")}</button>
      <button id="TabsInspector" class="compact" variant="ghost">{t("screen.creator-tools.inspector")}</button>
      <button id="TabsCatalogue" class="compact" variant="ghost">{t("screen.creator-tools.catalogue")}</button>
    </row>
    <column id="InspectorPanel" class:chosen={$panel == 'inspector'}>
      <text id="Selected">{$selected}</text>
      <scroll id="Scroll">
        <column id="ScrollInspector">{@render scrollInspector?.()}</column>
      </scroll>
    </column>
    <column id="CataloguePanel" class:chosen={$panel == 'catalogue'}>
      <text id="CataloguePanelTitle">{t("screen.creator-tools.catalogue")}</text>
      <column id="CataloguePanelCatalogue">{@render cataloguePanelCatalogue?.()}</column>
    </column>
  </grid>
</column>

<style>
.compact, .compact::part(base) { height: 2rem; }
.compact::part(base) { padding: 0 calc(6rem / 7); }
button.compact[variant="ghost"]::part(base) { padding-inline: calc(6rem / 7 - 1px); }
@media (pointer: coarse) {
.compact, .compact::part(base) { height: calc(22rem / 7); }
}
@media (-og-input: pad) {
.compact, .compact::part(base) { height: calc(20rem / 7); }
}
@media (width < 600px) and (orientation: portrait) {
button, button.compact, button::part(base), button.compact::part(base) { height: calc(16rem / 7); }
button::part(base), button.compact::part(base) { padding: 0 calc(8rem / 7); }
button.compact[variant="ghost"]::part(base) { padding-inline: calc(8rem / 7 - 1px); }
}
@media (width < 600px) and (orientation: portrait) and (pointer: coarse) {
button, button.compact, button::part(base), button.compact::part(base) { height: calc(22rem / 7); }
}
@media (width < 600px) and (orientation: portrait) and (-og-input: pad) {
button, button.compact, button::part(base), button.compact::part(base) { height: calc(24rem / 7); }
}
:root { --surface-saturation: 120%; }
@media (prefers-color-scheme: light) { :root { --surface-saturation: 200%; } }
#Editor { width: stretch; height: stretch; }
#Toolbar { width: stretch; padding: 8px; gap: 6px; align-items: center;  border-radius: var(--radius-panel); backdrop-filter: blur(32px) saturate(var(--surface-saturation)); -og-material: glass thick lift; flex-wrap: wrap; }
@media (width < 600px) and (orientation: portrait) {
}
#Select.selection-off { opacity: 0.85; }
#Select.selection-on { opacity: 1; }
@media (width < 600px) and (orientation: portrait) {
}
#Move.selection-off { opacity: 0.85; }
#Move.selection-on { opacity: 1; }
@media (width < 600px) and (orientation: portrait) {
}
#Rotate.selection-off { opacity: 0.85; }
#Rotate.selection-on { opacity: 1; }
@media (width < 600px) and (orientation: portrait) {
}
#Scale.selection-off { opacity: 0.85; }
#Scale.selection-on { opacity: 1; }
@media (width < 600px) and (orientation: portrait) {
}
@media (width < 600px) and (orientation: portrait) {
}
@media (width < 600px) and (orientation: portrait) {
}
#Play.run-stopped { opacity: 1; }
#Play.run-playing { opacity: 0.8; }
@media (width < 600px) and (orientation: portrait) {
}
#Body { width: stretch; height: stretch; grid-template-columns: 260px 1fr 320px; grid-template-rows: 1fr 200px; grid-template-areas: "tree view inspector" "tree catalogue inspector"; }
@media (600px <= width < 1024px) and (height >= 600px) {
#Body { grid-template-columns: 220px 1fr 280px; }
}
@media (width < 600px) and (orientation: portrait) {
#Body { grid-template-columns: 1fr; grid-template-rows: 1fr auto 260px; grid-template-areas: "view" "tabs" "panel"; }
}
#TreePanel { width: stretch; height: stretch; padding: 8px; gap: 6px;  border-radius: var(--radius-panel); backdrop-filter: blur(32px) saturate(var(--surface-saturation)); -og-material: glass thick lift; grid-area: tree; }
@media (width < 600px) and (orientation: portrait) {
#TreePanel { grid-area: panel; }
}
#TreePanelTitle { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#TreePanelHierarchy { width: stretch; height: stretch; }
#View { width: stretch; height: stretch; grid-area: view; }
#Tabs { width: stretch; gap: 6px; padding: 6px; grid-area: tabs; }
@media (width < 600px) and (orientation: portrait) {
}
@media (width < 600px) and (orientation: portrait) {
}
@media (width < 600px) and (orientation: portrait) {
}
@media (width < 600px) and (orientation: portrait) {
}
#InspectorPanel { width: stretch; height: stretch; padding: 8px; gap: 6px;  border-radius: var(--radius-panel); backdrop-filter: blur(32px) saturate(var(--surface-saturation)); -og-material: glass thick lift; grid-area: inspector; }
@media (width < 600px) and (orientation: portrait) {
#InspectorPanel { grid-area: panel; }
}
#Selected { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); line-clamp: 1; text-overflow: ellipsis; }
#Scroll { width: stretch; height: stretch; }
#ScrollInspector { width: stretch; }
#CataloguePanel { width: stretch; height: stretch; padding: 8px; gap: 6px;  border-radius: var(--radius-panel); backdrop-filter: blur(32px) saturate(var(--surface-saturation)); -og-material: glass thick lift; grid-area: catalogue; }
@media (width < 600px) and (orientation: portrait) {
#CataloguePanel { grid-area: panel; }
}
#CataloguePanelTitle { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#CataloguePanelCatalogue { width: stretch; height: stretch; }
@media (width < 600px) and (orientation: portrait) {
#Tabs { display: initial; }
#TreePanel, #InspectorPanel, #CataloguePanel { display: none; }
#TreePanel.chosen, #InspectorPanel.chosen, #CataloguePanel.chosen { display: initial; }
}
</style>
