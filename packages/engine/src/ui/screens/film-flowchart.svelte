<meta name="screen" content="film-flowchart" data-layer="menu" data-focus="Body/Graph" data-under="dim" data-input="menu" data-pause="true" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./film-flowchart.en.ogt">
<meta name="parts" content="film-flowchart.frame film-flowchart.band film-flowchart.title film-flowchart.detail film-flowchart.shade film-flowchart.kicker">
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

<div id="Screen" part="film-flowchart.frame" class="bare">
  <div id="Veil" part="film-flowchart.shade"></div>
  <header id="Header" part="film-flowchart.band">
    <button id="Back" variant="quiet" onclick={back}><div class="back"><og-icon class="chevron" icon="chevronLeft"></og-icon><span>{t("screen.common.back")}</span></div></button>
    <h1 id="Title" part="film-flowchart.title">{t("screen.film-flowchart.title")}</h1>
  </header>
  <div id="Body">
    <NodeGraph id="Graph" nodes={nodes} links={links} view={view} zoom={zoom} panX={panX} panY={panY} selected={selected} name={t("screen.film-flowchart.title")} />
  </div>
  {#if detail != ""}
    <div id="Detail" part="film-flowchart.detail">
      <div id="Words">
        <p id="Kicker" part="film-flowchart.kicker">{t("screen.film-flowchart.selected")}</p>
        <p id="What">{detail}</p>
      </div>
      <button id="Play" variant="primary" class="plate go" disabled={!canPlay} onclick={play}>{t("screen.film-flowchart.play")}</button>
    </div>
  {/if}
</div>

<style>
/* Over the dimmed film: a veil; the band (Back, the title); the chart; and
   the picked node on a plate lit in the accent, its words whole beside
   Play from here, which drops under them where the plate is narrow. */
:root { position: relative; display: flex; flex-direction: column; row-gap: var(--space-lg); block-size: stretch; overflow: hidden; padding: var(--space-xxl) var(--space-3xl) var(--space-xl); color: var(--front-ink); }
h1, p { margin: 0; }
[part="film-flowchart.shade"] { position: absolute; inset: 0; background: var(--system-veil); }
[part="film-flowchart.band"], #Body { position: relative; }
[part="film-flowchart.band"] { display: flex; align-items: baseline; column-gap: var(--space-lg); padding: 0; border: 0; min-block-size: 0; background: none; }
#Back { --button-ink: var(--front-ink-dim); --button-quiet-ink: var(--front-ink-dim); --button-quiet-hover-ink: var(--front-ink); flex: none; }
.back { display: flex; align-items: center; column-gap: var(--space-xs); }
.back .chevron { inline-size: var(--size-xs); block-size: var(--size-xs); }
.chevron:dir(rtl) { scale: -1 1; }
[part="film-flowchart.title"] { flex: 1 1 auto; min-inline-size: 0; font-size: var(--front-title-size); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); font-stretch: var(--front-stretch); letter-spacing: var(--font-tracking-xs); color: var(--front-ink); text-shadow: var(--front-shadow); overflow-wrap: break-word; }
#Body { flex: 1; min-block-size: 0; overflow: auto; }
[part="film-flowchart.detail"] { position: relative; flex: none; display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-sm) var(--space-lg); padding: var(--space-md) var(--space-lg); border-radius: 0; background: var(--dialog-fill); box-shadow: var(--hud-plate-lit-rim); }
#Words { flex: 1 1 calc(200 * var(--font-step-px)); min-inline-size: 0; display: flex; flex-direction: column; row-gap: var(--space-xxs); }
[part="film-flowchart.kicker"] { font-size: var(--hud-label-size); font-weight: var(--font-weight-bold); letter-spacing: var(--hud-label-tracking); text-transform: uppercase; color: var(--front-accent); }
[part="film-flowchart.kicker"]:lang(zh, ja, ko) { font-size: calc(14 * var(--font-step-px)); letter-spacing: 0; }
#What { font-size: var(--film-detail-size); line-height: var(--font-leading-snug); font-weight: var(--font-weight-semibold); color: var(--front-ink); overflow-wrap: break-word; }
#Play { flex: 0 1 auto; margin-inline-start: auto; max-inline-size: 100%; min-inline-size: calc(180 * var(--font-step-px)); min-block-size: var(--touch-target); white-space: normal; }
#Play::part(base) { padding-block: var(--button-wrap-pad, var(--space-sm)); }
button.plate { --button-radius: 0; --button-focus-radius: 0; --button-border-width: 0; --button-font-size: var(--dialog-label-size); --button-tracking: var(--font-tracking-lg); --button-transform: uppercase; }
button.plate::part(base) { border-width: 0; border-radius: var(--dialog-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
/* Chinese, Japanese and Korean have no capitals to track: the label is set a step larger instead. */
button.plate:lang(zh, ja, ko) { --button-font-size: var(--dialog-label-cjk-size); --button-tracking: 0; }
@media (width < 600px), (height < 600px) {
  :root { row-gap: var(--space-md); padding: var(--space-lg) calc(env(safe-area-inset-right) + var(--space-lg)) calc(env(safe-area-inset-bottom) + var(--space-md)) calc(env(safe-area-inset-left) + var(--space-lg)); }
  [part="film-flowchart.band"] { flex-wrap: wrap; row-gap: var(--space-xs); }
  #Back { margin-inline-start: calc(var(--space-sm) * -1); }
  [part="film-flowchart.title"] { flex: 1 0 100%; font-size: var(--front-title-size-compact); }
  #Play { flex: 1 1 100%; }
}
</style>
