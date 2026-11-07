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
    <button id="Select" variant="ghost" size="small" class:selection-off={!(($tool == "select"))} class:selection-on={($tool == "select")}>{t("screen.creator-tools.select")}</button>
    <button id="Move" variant="ghost" size="small" class:selection-off={!(($tool == "move"))} class:selection-on={($tool == "move")}>{t("screen.creator-tools.move")}</button>
    <button id="Rotate" variant="ghost" size="small" class:selection-off={!(($tool == "rotate"))} class:selection-on={($tool == "rotate")}>{t("screen.creator-tools.rotate")}</button>
    <button id="Scale" variant="ghost" size="small" class:selection-off={!(($tool == "scale"))} class:selection-on={($tool == "scale")}>{t("screen.creator-tools.scale")}</button>
    <spacer id="Node6"></spacer>
    <button id="Undo" size="small" variant="quiet" onclick={undo}>{t("screen.creator-tools.undo")}</button>
    <button id="Redo" size="small" variant="quiet" onclick={redo}>{t("screen.creator-tools.redo")}</button>
    <button id="Play" size="small" variant="primary" onclick={play} class:run-stopped={!($playing)} class:run-playing={$playing}>{t("screen.creator-tools.play")}</button>
    <button id="Save" size="small" disabled={!($dirty)} onclick={save}>{t("screen.common.save")}</button>
  </row>
  <grid id="Body">
    <column id="TreePanel" class:chosen={$panel == 'hierarchy'}>
      <text id="TreePanelTitle">{t("screen.creator-tools.hierarchy")}</text>
      <column id="TreePanelHierarchy">{@render treePanelHierarchy?.()}</column>
    </column>
    <column id="View">{@render view?.()}</column>
    <row id="Tabs" visible="false">
      <button id="TabsHierarchy" size="small" variant="ghost">{t("screen.creator-tools.hierarchy")}</button>
      <button id="TabsInspector" size="small" variant="ghost">{t("screen.creator-tools.inspector")}</button>
      <button id="TabsCatalogue" size="small" variant="ghost">{t("screen.creator-tools.catalogue")}</button>
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
#Editor { width: stretch; height: stretch; }
#Toolbar { width: stretch; padding: 8px; gap: 6px; align-items: center; background: var(--panel); -og-frame: panel; flex-wrap: wrap; }
@media phonePortrait {
}
#Select.selection-off { opacity: 0.85; }
#Select.selection-on { opacity: 1; }
@media phonePortrait {
}
#Move.selection-off { opacity: 0.85; }
#Move.selection-on { opacity: 1; }
@media phonePortrait {
}
#Rotate.selection-off { opacity: 0.85; }
#Rotate.selection-on { opacity: 1; }
@media phonePortrait {
}
#Scale.selection-off { opacity: 0.85; }
#Scale.selection-on { opacity: 1; }
@media phonePortrait {
}
@media phonePortrait {
}
@media phonePortrait {
}
#Play.run-stopped { opacity: 1; }
#Play.run-playing { opacity: 0.8; }
@media phonePortrait {
}
#Body { width: stretch; height: stretch; grid-template-columns: 260px 1fr 320px; grid-template-rows: 1fr 200px; grid-template-areas: "tree view inspector" "tree catalogue inspector"; }
@media tablet {
#Body { grid-template-columns: 220px 1fr 280px; }
}
@media phonePortrait {
#Body { grid-template-columns: 1fr; grid-template-rows: 1fr auto 260px; grid-template-areas: "view" "tabs" "panel"; }
}
#TreePanel { width: stretch; height: stretch; padding: 8px; gap: 6px; background: var(--panel); -og-frame: panel; grid-area: tree; }
@media phonePortrait {
#TreePanel { grid-area: panel; }
}
#TreePanelTitle { font: var(--text-caption); color: var(--ink-muted); }
#TreePanelHierarchy { width: stretch; height: stretch; }
#View { width: stretch; height: stretch; grid-area: view; }
#Tabs { width: stretch; gap: 6px; padding: 6px; grid-area: tabs; }
@media phonePortrait {
}
@media phonePortrait {
}
@media phonePortrait {
}
@media phonePortrait {
}
#InspectorPanel { width: stretch; height: stretch; padding: 8px; gap: 6px; background: var(--panel); -og-frame: panel; grid-area: inspector; }
@media phonePortrait {
#InspectorPanel { grid-area: panel; }
}
#Selected { font: var(--text-lg); line-clamp: 1; text-overflow: ellipsis; }
#Scroll { width: stretch; height: stretch; }
#ScrollInspector { width: stretch; }
#CataloguePanel { width: stretch; height: stretch; padding: 8px; gap: 6px; background: var(--panel); -og-frame: panel; grid-area: catalogue; }
@media phonePortrait {
#CataloguePanel { grid-area: panel; }
}
#CataloguePanelTitle { font: var(--text-caption); color: var(--ink-muted); }
#CataloguePanelCatalogue { width: stretch; height: stretch; }
@media phonePortrait {
button { -og-control-size: medium; }
#Tabs { display: initial; }
#TreePanel, #InspectorPanel, #CataloguePanel { display: none; }
#TreePanel.chosen, #InspectorPanel.chosen, #CataloguePanel.chosen { display: initial; }
}
</style>
