<meta name="screen" content="film-flowchart" data-layer="menu" data-focus="Body/Graph" data-under="dim" data-input="menu" data-pause="true" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./film-flowchart.en.ogt">
<meta name="parts" content="film-flowchart.frame film-flowchart.band film-flowchart.title film-flowchart.detail">
<meta name="prop:nodes" content="list" data-doc="og.film.chart's nodes as NodeGraph's: keyed rows of { id (the node), x, y, label, icon, state, shape, team }. state: current (where the film is), done (a segment seen), locked (not seen); an ending's shape diamond, a chapter's square, the rest circle.">
<meta name="prop:links" content="list" data-doc="og.film.chart's links as NodeGraph's: keyed rows of { id, d, active }, active a choice the player has taken.">
<meta name="prop:view" content="string" data-default="0 0 600 400" data-doc="The chart's extent, `0 0 width height` from og.film.chart.">
<meta name="prop:zoom" content="number" data-default="1" data-doc="NodeGraph's zoom, kept by the game from its zoom and pinch actions.">
<meta name="prop:panX" content="number" data-doc="NodeGraph's pan across.">
<meta name="prop:panY" content="number" data-doc="NodeGraph's pan down.">
<meta name="prop:selected" content="string" data-doc="The node picked (its id); empty is none.">
<meta name="prop:detail" content="string" data-doc="What the picked node is, as the game words it (its chapter, or Not seen yet); empty hides the panel.">
<meta name="prop:canPlay" content="bool" data-doc="The picked node can be played from (a chapter the player has saved at).">
<meta name="action:pick" content="Pick a node">
<meta name="action:play" content="Play from here">
<meta name="action:back" content="Back">
<meta name="fixture:default" content='{"view":"0 0 720 360","zoom":1,"panX":0,"panY":0,"selected":"hall","detail":"Chapter 2 · The hall","canPlay":true,"links":[{"id":"a","d":"M60 60 C 160 60, 160 60, 260 60","active":true},{"id":"b","d":"M60 60 C 160 60, 160 180, 260 180","active":false},{"id":"c","d":"M260 60 C 360 60, 360 60, 460 60","active":true},{"id":"d","d":"M260 60 C 360 60, 360 180, 460 180","active":false},{"id":"e","d":"M460 60 C 560 60, 560 60, 660 60","active":false}],"nodes":[{"id":"start","x":60,"y":60,"label":"Kitchen","icon":"flag","state":"done","shape":"square","team":""},{"id":"hall","x":260,"y":60,"label":"Hall","icon":"play","state":"done","shape":"square","team":""},{"id":"wait","x":260,"y":180,"label":"Wait","icon":"clock","state":"locked","shape":"circle","team":""},{"id":"brave","x":460,"y":60,"label":"Stairs","icon":"play","state":"current","shape":"circle","team":""},{"id":"shy","x":460,"y":180,"label":"Cellar","icon":"lock","state":"locked","shape":"circle","team":""},{"id":"end","x":660,"y":60,"label":"Dawn","icon":"crown","state":"locked","shape":"diamond","team":""}]}'>

<column id="Screen" part="film-flowchart.frame" class="bare">
  <row id="Header" part="film-flowchart.band">
    <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
    <text id="Title" part="film-flowchart.title" heading="true">{t("screen.film-flowchart.title")}</text>
  </row>
  <scroll id="Body">
    <NodeGraph id="Graph" nodes={$nodes} links={$links} view={$view} zoom={$zoom} panX={$panX} panY={$panY} selected={$selected} name={t("screen.film-flowchart.title")} />
  </scroll>
  <row id="Detail" part="film-flowchart.detail" hidden={$detail == ""}>
    <text id="What">{$detail}</text>
    <button id="Play" variant="primary" disabled={!($canPlay)} onclick={play}>{t("screen.film-flowchart.play")}</button>
  </row>
</column>

<style>
#Screen { width: stretch; height: stretch; padding: var(--space-xxl); gap: var(--space-lg); }
#Header { width: stretch; gap: var(--space-md); align-items: center; }
#Title { font-size: var(--type-xl2-size); }
#Body { width: stretch; height: stretch; min-height: 0; }
#Detail { width: stretch; gap: var(--space-md); align-items: center; justify-content: space-between; padding: var(--space-md); background: var(--hud); border-radius: var(--radius-card); }
#What { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); line-clamp: 1; text-overflow: ellipsis; }
@media (width < 600px), (height < 600px) {
#Screen { padding: var(--space-lg) calc(env(safe-area-inset-right) + var(--space-md)) calc(env(safe-area-inset-bottom) + var(--space-lg)) calc(env(safe-area-inset-left) + var(--space-md)); gap: var(--space-md); }
#Title { font-size: var(--type-xl-size); }
}
</style>
