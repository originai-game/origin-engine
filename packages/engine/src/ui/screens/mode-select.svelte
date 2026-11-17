<meta name="screen" content="mode-select" data-layer="menu" data-focus="Modes" data-under="dim" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./mode-select.en.ogt">
<meta name="parts" content="mode-select.frame mode-select.band mode-select.title mode-select.cell mode-select.name">
<meta name="prop:modes" content="list" data-doc="The modes: { name, line, tag, locked, reason, texture }.">
<meta name="prop:selected" content="string" data-doc="The stable collection key selected now.">
<meta name="action:confirm" content="Play">
<meta name="action:back" content="Back">
<meta name="fixture:default" content="{&quot;modes&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;line&quot;:&quot;The campaign, alone or with a friend.&quot;,&quot;locked&quot;:false,&quot;name&quot;:&quot;Story&quot;,&quot;reason&quot;:&quot;&quot;,&quot;tag&quot;:&quot;&quot;,&quot;texture&quot;:0},{&quot;id&quot;:&quot;item-1&quot;,&quot;line&quot;:&quot;Ranked fights, one life.&quot;,&quot;locked&quot;:false,&quot;name&quot;:&quot;Arena&quot;,&quot;reason&quot;:&quot;&quot;,&quot;tag&quot;:&quot;Ranked&quot;,&quot;texture&quot;:0},{&quot;id&quot;:&quot;item-2&quot;,&quot;line&quot;:&quot;Hard challenges for the brave.&quot;,&quot;locked&quot;:true,&quot;name&quot;:&quot;Trials&quot;,&quot;reason&quot;:&quot;Finish chapter 2&quot;,&quot;tag&quot;:&quot;New&quot;,&quot;texture&quot;:0}],&quot;selected&quot;:&quot;item-0&quot;}">

<div id="Screen" part="mode-select.frame" class="bare">
  <header id="Header" part="mode-select.band">
    <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
    <h1 id="Title" part="mode-select.title">{t("screen.mode-select.title")}</h1>
  </header>
  <div id="Scroll">
    <div id="Modes" focus-group="wrap">
      {#each modes as mode (mode.id)}
        <article id="Mode" part="mode-select.cell" interactive="true" focusable="true" onclick={select} class:selection-off={mode.id != selected} class:selection-on={mode.id == selected} class:lock-open={!mode.locked} class:lock-locked={mode.locked}>
          <art id="Art" kind="levelThumb" src={mode.texture}></art>
          <div id="Head">
            <h2 id="Name" part="mode-select.name">{mode.name}</h2>
            {#if mode.tag != ""}<badge id="Tag" tone="accent">{mode.tag}</badge>{/if}
          </div>
          <p id="Line">{mode.line}</p>
          {#if mode.locked}<p id="Reason">{mode.reason}</p>{/if}
        </article>
      {/each}
    </div>
  </div>
  <div id="Detail">{@render detail?.()}</div>
  <footer id="Footer" data-layout="cluster">
    <button id="Play" variant="primary" disabled={modes.length == 0} onclick={play} onconfirm={play}>{t("screen.mode-select.play")}</button>
  </footer>
</div>

<style>
:root { display: flex; flex-direction: column; gap: var(--space-xl); block-size: stretch; padding: var(--space-xxl); }
h1, h2, p { margin: 0; }
[part="mode-select.band"] { display: flex; align-items: center; gap: var(--space-md); }
[part="mode-select.title"] { font-size: var(--type-xl2-size); letter-spacing: var(--type-xl2-tracking); }
#Scroll { flex: 1; min-block-size: 0; overflow-y: auto; }
/* The modes as cards: three across, two on a tablet, one on a phone. */
#Modes { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--space-lg); }
[part="mode-select.cell"] { display: flex; flex-direction: column; gap: var(--space-sm); padding: var(--space-md); }
[part="mode-select.cell"]:hover, [part="mode-select.cell"]:focus-visible { translate: 0 var(--state-lift); }
[part="mode-select.cell"]:active { translate: 0 0; }
/* An unselected or locked card dims its art; its words keep their ink. */
.selection-off #Art { opacity: var(--opacity-soft); }
.lock-locked #Art { opacity: var(--opacity-dim); filter: grayscale(1); }
#Art { inline-size: 100%; aspect-ratio: 1.78; }
#Head { display: flex; align-items: center; gap: var(--space-sm); }
[part="mode-select.name"] { flex: 1; font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#Line { color: var(--ink-muted); line-clamp: 2; }
#Reason { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--warn); }
#Footer { justify-content: end; gap: var(--space-md); }
#Play, #Play::part(base) { block-size: var(--control-lg); }
@media (600px <= width < 1024px) and (height >= 600px) {
  #Modes { grid-template-columns: repeat(2, 1fr); }
}
@media (width < 600px), (height < 600px) {
  :root { padding: var(--space-lg); gap: var(--space-md); }
  [part="mode-select.title"] { font-size: var(--type-xl-size); letter-spacing: var(--type-xl-tracking); }
  #Play { flex: 1; }
}
@media (width < 600px) and (orientation: portrait) {
  #Modes { grid-template-columns: 1fr; gap: var(--space-md); }
}
</style>
