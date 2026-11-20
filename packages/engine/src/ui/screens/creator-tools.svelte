<meta name="screen" content="creator-tools" data-layer="menu" data-focus="" data-under="none" data-input="all" data-pause="false" data-capture="true" data-remember="true" data-back="exit">
<meta name="strings" content="./creator-tools.en.ogt">
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
  <div id="Toolbar" part="frame" variant="hud" role="toolbar">
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
    <section id="TreePanel" part="frame" class="panel" variant="hud" class:chosen={panel == 'hierarchy'}>
      <h2 class="label" id="TreePanelTitle" part="label">{t("screen.creator-tools.hierarchy")}</h2>
      <div id="TreePanelHierarchy" class="fill">{@render treePanelHierarchy?.()}</div>
    </section>
    <div id="View">{@render view?.()}</div>
    <div id="Tabs">
      <button id="TabsHierarchy" class="compact tab" variant="ghost" class:on={panel == "hierarchy"} aria-current={panel == "hierarchy"}>{t("screen.creator-tools.hierarchy")}</button>
      <button id="TabsInspector" class="compact tab" variant="ghost" class:on={panel == "inspector"} aria-current={panel == "inspector"}>{t("screen.creator-tools.inspector")}</button>
      <button id="TabsCatalogue" class="compact tab" variant="ghost" class:on={panel == "catalogue"} aria-current={panel == "catalogue"}>{t("screen.creator-tools.catalogue")}</button>
    </div>
    <section id="InspectorPanel" part="frame" class="panel" variant="hud" class:chosen={panel == 'inspector'}>
      <h2 class="title" id="Selected" part="title">{selected}</h2>
      <div id="Scroll" class="fill">
        <div id="ScrollInspector">{@render scrollInspector?.()}</div>
      </div>
    </section>
    <section id="CataloguePanel" part="frame" class="panel" variant="hud" class:chosen={panel == 'catalogue'}>
      <h2 class="label" id="CataloguePanelTitle" part="label">{t("screen.creator-tools.catalogue")}</h2>
      <div id="CataloguePanelCatalogue" class="fill">{@render cataloguePanelCatalogue?.()}</div>
    </section>
  </div>
</div>

<style>
#Editor { display: flex; flex-direction: column; block-size: stretch; color: var(--hud-ink); }
h2 { margin: 0; }
.compact, .compact { block-size: var(--control-sm); }
.compact { padding: 0 var(--space-md); }
button.compact { padding-inline: calc(var(--space-md) - var(--button-border-width)); }
#Toolbar { display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-sm); --plate-fill: var(--hud-plate-fill); --plate-shadow: var(--hud-plate-rim); --plate-radius: 0px; }
#Undo { margin-inline-start: auto; }
/* The tools: one segmented switch, the chosen tool washed in the accent
   with a lit foot; the others keep their words' ink and lose their rim. */
#Select, #Move, #Rotate, #Scale { --button-radius: 0px; --button-ghost-ink: var(--hud-ink-dim); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); }
#Select, #Move, #Rotate, #Scale { background: var(--data-well); box-shadow: var(--data-well-rim); }
#Toolbar button.selection-off { border-color: transparent; }
#Toolbar button.selection-on { background: rgb(from var(--hud-accent) r g b / 0.2); box-shadow: inset 0 calc(-2 * var(--stroke-hairline)) 0 var(--hud-accent), var(--data-well-rim); --button-ghost-ink: var(--hud-accent); }
#Toolbar button.selection-on { border-color: transparent; }
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
/* A panel's scroller fades out over its foot, where its content pads as much,
   so a control running past the edge reads as more below, never as cut, and
   the last one scrolled into view stands clear of the fade. */
#Scroll { overflow-y: auto; mask-image: linear-gradient(180deg, #000 calc(100% - var(--space-lg)), transparent); }
#ScrollInspector { padding-block-end: var(--space-lg); }
#TreePanel { grid-area: tree; }
#View { grid-area: view; }
#Tabs { grid-area: tabs; display: none; gap: var(--space-xs); padding: var(--space-sm); }
#InspectorPanel { grid-area: inspector; }
#CataloguePanel { grid-area: catalogue; }
.label { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--data-head-ink); }
.title { font-size: var(--hud-size-sm); font-weight: var(--data-title-weight); color: var(--hud-ink); line-clamp: 1; text-overflow: ellipsis; }
button:size(narrow), button.compact:size(narrow), button:size(narrow), button.compact:size(narrow) { block-size: var(--control-md); }
button:size(narrow), button.compact:size(narrow) { padding: 0 var(--space-lg); }
button.compact:size(narrow) { padding-inline: calc(var(--space-lg) - var(--button-border-width)); }
#Body:size(narrow) { grid-template-columns: minmax(0, 1fr); grid-template-rows: minmax(0, 2fr) auto minmax(var(--measure-sm), 3fr); grid-template-areas: "view" "tabs" "panel"; }
#Tabs:size(narrow) { display: flex; flex-wrap: nowrap; overflow-x: auto; overscroll-behavior-x: contain; scrollbar-width: none; min-inline-size: 0; background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); }
.tab:size(narrow), .tab.compact:size(narrow), .tab:size(narrow), .tab.compact:size(narrow) { block-size: var(--touch-target); }
.tab:size(narrow) { flex: none; scroll-margin-inline: var(--space-xl); white-space: nowrap; --button-radius: 0px; --button-ghost-ink: var(--hud-ink-dim); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); }
.tab:size(narrow) { border-color: transparent; }
.tab.on:size(narrow) { background: rgb(from var(--hud-accent) r g b / 0.2); box-shadow: inset 0 calc(-2 * var(--stroke-hairline)) 0 var(--hud-accent); --button-ghost-ink: var(--hud-accent); }
#TreePanel:size(narrow), #InspectorPanel:size(narrow), #CataloguePanel:size(narrow) { grid-area: panel; display: none; }
#TreePanel.chosen:size(narrow), #InspectorPanel.chosen:size(narrow), #CataloguePanel.chosen:size(narrow) { display: flex; }
</style>
