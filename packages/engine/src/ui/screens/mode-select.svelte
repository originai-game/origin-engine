<meta name="screen" content="mode-select" data-layer="menu" data-focus="Modes" data-under="dim" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./mode-select.en.ogt">
<meta name="prop:modes" content="list" data-doc="The modes: { name, line, tag, locked, reason, texture }.">
<meta name="prop:selected" content="string" data-doc="The stable collection key selected now.">
<meta name="action:confirm" content="Play">
<meta name="action:back" content="Back">
<meta name="fixture:default" content="{&quot;modes&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;line&quot;:&quot;The campaign, alone or with a friend.&quot;,&quot;locked&quot;:false,&quot;name&quot;:&quot;Story&quot;,&quot;reason&quot;:&quot;&quot;,&quot;tag&quot;:&quot;&quot;,&quot;texture&quot;:0},{&quot;id&quot;:&quot;item-1&quot;,&quot;line&quot;:&quot;Ranked fights, one life.&quot;,&quot;locked&quot;:false,&quot;name&quot;:&quot;Arena&quot;,&quot;reason&quot;:&quot;&quot;,&quot;tag&quot;:&quot;Ranked&quot;,&quot;texture&quot;:0},{&quot;id&quot;:&quot;item-2&quot;,&quot;line&quot;:&quot;Hard challenges for the brave.&quot;,&quot;locked&quot;:true,&quot;name&quot;:&quot;Trials&quot;,&quot;reason&quot;:&quot;Finish chapter 2&quot;,&quot;tag&quot;:&quot;New&quot;,&quot;texture&quot;:0}],&quot;selected&quot;:&quot;item-0&quot;}">

<column id="Screen">
  <row id="Header">
    <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
    <text id="Title" heading="true">{t("screen.mode-select.title")}</text>
  </row>
  <scroll id="Scroll">
    <grid id="Modes" focus-group="wrap">
      {#each $modes as mode (mode.id)}
        <column id="Mode" interactive="true" focusable="true" onclick={select} class:selection-off={!((mode.id == $selected))} class:selection-on={(mode.id == $selected)} class:lock-open={!(mode.locked)} class:lock-locked={mode.locked}>
          <art id="Art" kind="levelThumb" src={mode.texture}></art>
          <row id="Head">
            <text id="Name" heading="true">{mode.name}</text>
            <spacer id="Node11"></spacer>
            <badge id="Tag" tone="accent" hidden={!((mode.tag != ""))}>{mode.tag}</badge>
          </row>
          <text id="Line">{mode.line}</text>
          <text id="Reason" hidden={!(mode.locked)}>{mode.reason}</text>
        </column>
      {/each}
    </grid>
  </scroll>
  <column id="Detail">{@render detail?.()}</column>
  <row id="Footer">
    <button id="Play" variant="primary" size="large" disabled={($modes.length == 0.0)} onclick={play} onconfirm={play}>{t("screen.mode-select.play")}</button>
  </row>
</column>

<style>
#Screen { width: stretch; height: stretch; padding: 32px; gap: 20px; }
@media (width < 600px), (height < 600px) {
#Screen { padding: 16px; gap: 12px; }
}
#Header { width: stretch; gap: 12px; align-items: center; }
#Title { font: var(--text-xl2); }
@media (width < 600px), (height < 600px) {
#Title { font: var(--text-xl); }
}
#Scroll { width: stretch; height: stretch; }
#Modes { width: stretch; gap: 16px; grid-template-columns: 1fr 1fr 1fr; }
@media (600px <= width < 1024px) and (height >= 600px) {
#Modes { grid-template-columns: 1fr 1fr; }
}
@media (width < 600px) and (orientation: portrait) {
#Modes { grid-template-columns: 1fr; gap: 12px; }
}
#Mode { width: stretch; padding: 12px; gap: 8px; background: var(--panel-raised); -og-frame: card; }
#Mode.selection-off { opacity: 0.85; }
#Mode.selection-on { opacity: 1; }
#Mode.lock-open { scale: 1; }
#Mode.lock-locked { opacity: 0.55; }
#Mode { translate: 0 0; }
#Mode:hover { translate: 0 -4px; }
#Mode:active { translate: 0 0; }
#Mode:focus-visible { translate: 0 -4px; }
#Art { width: stretch; aspect-ratio: 1.78; }
#Head { width: stretch; gap: 8px; align-items: center; }
#Name { font: var(--text-lg); }
#Line { color: var(--ink-muted); line-clamp: 2; }
#Reason { font: var(--text-caption); color: var(--warn); }
#Detail { width: stretch; }
#Footer { width: stretch; gap: 12px; justify-content: end; }
@media (width < 600px), (height < 600px) {
#Play { width: stretch; }
}
</style>
