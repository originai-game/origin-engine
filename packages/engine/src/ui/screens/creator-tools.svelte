<meta name="screen" content="creator-tools" data-layer="menu" data-focus="" data-under="none" data-input="all" data-pause="false" data-capture="true" data-remember="true" data-back="exit">
<meta name="strings" content="./creator-tools.en.ogt">
<meta name="parts" content="creator-tools.frame creator-tools.label creator-tools.title">
<meta name="prop:tool" content="string" data-doc="select, move, rotate or scale.">
<meta name="prop:playing" content="bool" data-doc="The world is playing.">
<meta name="prop:selected" content="string" data-doc="The selected object's name (empty: none).">
<meta name="prop:panel" content="string" data-doc="On a phone, the open panel: hierarchy, inspector or catalogue.">
<meta name="prop:dirty" content="bool" data-doc="Unsaved changes.">
<meta name="action:back" content="Exit">
<meta name="fixture:default" content="{&quot;dirty&quot;:true,&quot;panel&quot;:&quot;inspector&quot;,&quot;playing&quot;:false,&quot;selected&quot;:&quot;Lantern_03&quot;,&quot;tool&quot;:&quot;move&quot;}">

<!-- The creator's tools on the HUD's smoked glass: the four tools a
     segmented switch lit in the accent, the panels plates with tracked
     capital titles. On a phone the panel tabs are one strip that scrolls
     sideways, each a finger's target; a focus move scrolls its tab into view. -->
<div id="Editor">
  <div id="Toolbar" part="creator-tools.frame" class="hud" role="toolbar">
    <button id="Select" variant="ghost" class="compact" class:selection-off={tool != "select"} class:selection-on={tool == "select"}>{t("screen.creator-tools.select")}</button>
    <button id="Move" variant="ghost" class="compact" class:selection-off={tool != "move"} class:selection-on={tool == "move"}>{t("screen.creator-tools.move")}</button>
    <button id="Rotate" variant="ghost" class="compact" class:selection-off={tool != "rotate"} class:selection-on={tool == "rotate"}>{t("screen.creator-tools.rotate")}</button>
    <button id="Scale" variant="ghost" class="compact" class:selection-off={tool != "scale"} class:selection-on={tool == "scale"}>{t("screen.creator-tools.scale")}</button>
    <button id="Undo" class="compact" variant="quiet" onclick={undo}>{t("screen.creator-tools.undo")}</button>
    <button id="Redo" class="compact" variant="quiet" onclick={redo}>{t("screen.creator-tools.redo")}</button>
    <button id="Play" class="compact" variant="primary" onclick={play} class:run-stopped={!playing} class:run-playing={playing}>{t("screen.creator-tools.play")}</button>
    <button id="Save" class="compact" disabled={!dirty} onclick={save}>{t("screen.common.save")}</button>
  </div>
  <div id="Body">
    <section id="TreePanel" part="creator-tools.frame" class="hud panel" class:chosen={panel == 'hierarchy'}>
      <h2 id="TreePanelTitle" part="creator-tools.label">{t("screen.creator-tools.hierarchy")}</h2>
      <div id="TreePanelHierarchy" class="fill">{@render treePanelHierarchy?.()}</div>
    </section>
    <div id="View">{@render view?.()}</div>
    <div id="Tabs">
      <button id="TabsHierarchy" class="compact tab" variant="ghost" class:on={panel == "hierarchy"} aria-current={panel == "hierarchy"}>{t("screen.creator-tools.hierarchy")}</button>
      <button id="TabsInspector" class="compact tab" variant="ghost" class:on={panel == "inspector"} aria-current={panel == "inspector"}>{t("screen.creator-tools.inspector")}</button>
      <button id="TabsCatalogue" class="compact tab" variant="ghost" class:on={panel == "catalogue"} aria-current={panel == "catalogue"}>{t("screen.creator-tools.catalogue")}</button>
    </div>
    <section id="InspectorPanel" part="creator-tools.frame" class="hud panel" class:chosen={panel == 'inspector'}>
      <h2 id="Selected" part="creator-tools.title">{selected}</h2>
      <div id="Scroll" class="fill">
        <div id="ScrollInspector">{@render scrollInspector?.()}</div>
      </div>
    </section>
    <section id="CataloguePanel" part="creator-tools.frame" class="hud panel" class:chosen={panel == 'catalogue'}>
      <h2 id="CataloguePanelTitle" part="creator-tools.label">{t("screen.creator-tools.catalogue")}</h2>
      <div id="CataloguePanelCatalogue" class="fill">{@render cataloguePanelCatalogue?.()}</div>
    </section>
  </div>
</div>

<style>
:root { display: flex; flex-direction: column; block-size: stretch; color: var(--hud-ink); }
h2 { margin: 0; }
.compact, .compact::part(base) { block-size: var(--control-sm); }
.compact::part(base) { padding: 0 var(--space-md); }
button.compact[variant="ghost"]::part(base) { padding-inline: calc(var(--space-md) - var(--button-border-width)); }
#Toolbar { display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-sm); --plate-fill: var(--hud-plate-fill); --plate-shadow: var(--hud-plate-rim); --plate-radius: 0px; }
#Undo { margin-inline-start: auto; }
/* The tools: one segmented switch, the chosen tool washed in the accent
   with a lit foot; the others keep their words' ink and lose their rim. */
#Select, #Move, #Rotate, #Scale { --button-radius: 0px; --button-ghost-ink: var(--hud-ink-dim); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); }
#Select, #Move, #Rotate, #Scale { background: var(--data-well); box-shadow: var(--data-well-rim); }
#Toolbar button.selection-off::part(base) { border-color: transparent; }
#Toolbar button.selection-on { background: rgb(from var(--hud-accent) r g b / 0.2); box-shadow: inset 0 calc(-2 * var(--stroke-hairline)) 0 var(--hud-accent), var(--data-well-rim); --button-ghost-ink: var(--hud-accent); }
#Toolbar button.selection-on::part(base) { border-color: transparent; }
#Play.run-playing { opacity: 0.8; }
/* The hierarchy, the view and the inspector side by side, the catalogue
   under the view; on a portrait phone the view over one chosen panel. */
#Body { flex: 1; min-block-size: 0; display: grid; grid-template-columns: var(--measure-sm) 1fr var(--measure-md); grid-template-rows: 1fr var(--measure-xs); grid-template-areas: "tree view inspector" "tree catalogue inspector"; }
/* A panel is as wide as its column, never as wide as its longest word: its
   title ends in an ellipsis instead of pushing the panel off the screen. */
.panel { display: flex; flex-direction: column; gap: var(--space-sm); min-block-size: 0; min-inline-size: 0; --plate-fill: var(--hud-plate-fill); --plate-shadow: var(--hud-plate-rim); --plate-radius: 0px; }
/* A lit seam between the view and a panel beside it, as a split's divider. */
#TreePanel { box-shadow: var(--hud-plate-rim), inset calc(-1 * var(--stroke-thin)) 0 0 rgb(from var(--hud-accent) r g b / 0.35); }
#InspectorPanel { box-shadow: var(--hud-plate-rim), inset var(--stroke-thin) 0 0 rgb(from var(--hud-accent) r g b / 0.35); }
.fill { flex: 1; min-block-size: 0; }
#Scroll { overflow-y: auto; }
#TreePanel { grid-area: tree; }
#View { grid-area: view; }
#Tabs { grid-area: tabs; display: none; gap: var(--space-xs); padding: var(--space-sm); }
#InspectorPanel { grid-area: inspector; }
#CataloguePanel { grid-area: catalogue; }
[part="creator-tools.label"] { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--data-head-ink); }
[part="creator-tools.title"] { font-size: var(--hud-size-sm); font-weight: var(--data-title-weight); color: var(--hud-ink); line-clamp: 1; text-overflow: ellipsis; }
@media (600px <= width < 1024px) and (height >= 600px) {
  #Body { grid-template-columns: var(--measure-xs) 1fr var(--measure-sm); }
}
@media (width < 600px) and (orientation: portrait) {
  button, button.compact, button::part(base), button.compact::part(base) { block-size: var(--control-md); }
  button::part(base), button.compact::part(base) { padding: 0 var(--space-lg); }
  button.compact[variant="ghost"]::part(base) { padding-inline: calc(var(--space-lg) - var(--button-border-width)); }
  /* One column no wider than the screen (it shrinks below its content). */
  #Body { grid-template-columns: minmax(0, 1fr); grid-template-rows: 1fr auto var(--measure-sm); grid-template-areas: "view" "tabs" "panel"; }
  /* The tabs: one strip that scrolls sideways, never wrapping, each tab a
     finger's target that keeps its whole name. A focus move scrolls its
     tab into view with a neighbour's edge showing beside it. */
  #Tabs { display: flex; flex-wrap: nowrap; overflow-x: auto; overscroll-behavior-x: contain; scrollbar-width: none; min-inline-size: 0; background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); }
  .tab, .tab.compact, .tab::part(base), .tab.compact::part(base) { block-size: var(--touch-target); }
  .tab { flex: none; scroll-margin-inline: var(--space-xl); white-space: nowrap; --button-radius: 0px; --button-ghost-ink: var(--hud-ink-dim); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); }
  .tab::part(base) { border-color: transparent; }
  .tab.on { background: rgb(from var(--hud-accent) r g b / 0.2); box-shadow: inset 0 calc(-2 * var(--stroke-hairline)) 0 var(--hud-accent); --button-ghost-ink: var(--hud-accent); }
  #TreePanel, #InspectorPanel, #CataloguePanel { grid-area: panel; display: none; }
  #TreePanel.chosen, #InspectorPanel.chosen, #CataloguePanel.chosen { display: flex; }
}
</style>
