<meta name="screen" content="loading" data-layer="modal" data-focus="" data-under="hide" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./loading.en.ogt">
<meta name="prop:keyArt" content="number" data-doc="The key art's texture (og.load.texture; 0: the style's placeholder).">
<meta name="prop:place" content="string" data-doc="Where the player is going.">
<meta name="prop:tip" content="string" data-doc="A tip or a line of lore.">
<meta name="prop:progress" content="number" data-doc="0 to 1.">
<meta name="prop:ready" content="bool" data-doc="Loading is done.">
<meta name="prop:waits" content="bool" data-doc="The game waits for a press once ready.">
<meta name="action:confirm" content="Continue">
<meta name="fixture:default" content="{&quot;place&quot;:&quot;The Sunken Archive&quot;,&quot;progress&quot;:0.62,&quot;ready&quot;:false,&quot;tip&quot;:&quot;Dodging through an attack at the last moment slows time for a breath.&quot;,&quot;waits&quot;:true}">

<stack id="Loading" interactive="true" onclick={nextTip}>
  <art id="Art" kind="tutorial" src={$keyArt}></art>
  <column id="Scrim"></column>
  <column id="Foot">
    <spacer id="Node4"></spacer>
    <column id="Tip">
      <text id="Eyebrow">{t("screen.loading.tip")}</text>
      <text id="Text">{$tip}</text>
      <column id="Lore">{@render lore?.()}</column>
    </column>
    <text id="Place" heading="true">{$place}</text>
    <row id="Progress">
      <progress id="Bar" bind:value={$progress}></progress>
      <text id="Percent" hidden={!(!($ready))}>{t("screen.loading.Percent", { progress: formatPercent($progress) })}</text>
      <button id="Continue" variant="primary" hidden={!(($ready && $waits))} autofocus="true" onclick={continue} onconfirm={continue}>{t("screen.loading.continue")}</button>
    </row>
  </column>
</stack>

<style>
#Loading { width: stretch; height: stretch; }
#Art { object-fit: cover; width: stretch; height: stretch; position: absolute; inset-inline-start: 0; top: 0; inset-inline-end: 0; bottom: 0; }
#Scrim { width: stretch; height: stretch; position: absolute; inset-inline-start: 0; top: 0; inset-inline-end: 0; bottom: 0; background: #000000; opacity: 0.5; }
#Foot { width: stretch; height: stretch; padding: 48px; gap: 12px; }
@media phonePortrait phoneLandscape {
#Foot { padding: 24px; }
}
#Tip { width: stretch; max-width: 720px; padding: 16px; gap: 6px; background: var(--panel); -og-frame: panel; }
#Eyebrow { font: var(--text-caption); color: var(--primary); }
#Text { font: var(--text-reading); line-clamp: 4; }
#Lore { width: stretch; }
#Place { font: var(--text-xl2); }
@media phonePortrait phoneLandscape {
#Place { font: var(--text-lg); }
}
#Progress { width: stretch; gap: 12px; align-items: center; }
#Bar { width: stretch; height: 12px; }
#Percent { font: var(--text-caption); color: var(--ink-muted); }
</style>
