<meta name="screen" content="loading" data-layer="modal" data-focus="" data-under="hide" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./loading.en.ogt">
<meta name="parts" content="loading.scrim loading.frame loading.title">
<meta name="prop:keyArt" content="number" data-doc="The key art's texture (og.load.texture; 0: the style's placeholder).">
<meta name="prop:place" content="string" data-doc="Where the player is going.">
<meta name="prop:tip" content="string" data-doc="A tip or a line of lore.">
<meta name="prop:progress" content="number" data-doc="0 to 1.">
<meta name="prop:ready" content="bool" data-doc="Loading is done.">
<meta name="prop:waits" content="bool" data-doc="The game waits for a press once ready.">
<meta name="action:confirm" content="Continue">
<meta name="fixture:default" content="{&quot;place&quot;:&quot;The Sunken Archive&quot;,&quot;progress&quot;:0.62,&quot;ready&quot;:false,&quot;tip&quot;:&quot;Dodging through an attack at the last moment slows time for a breath.&quot;,&quot;waits&quot;:true}">

<div id="Loading" interactive="true" onclick={nextTip}>
  <art id="Art" kind="tutorial" src={keyArt}></art>
  <div id="Scrim" part="loading.scrim"></div>
  <div id="Foot">
    <section id="Tip" part="loading.frame">
      <p id="Eyebrow">{t("screen.loading.tip")}</p>
      <p id="Text">{tip}</p>
      <div id="Lore">{@render lore?.()}</div>
    </section>
    <h2 id="Place" part="loading.title">{place}</h2>
    <div id="Progress">
      <progress id="Bar" bind:value={progress}></progress>
      {#if !ready}<p id="Percent">{t("screen.loading.Percent", { progress: formatPercent(progress) })}</p>{/if}
      {#if ready && waits}<button id="Continue" variant="primary" autofocus="true" onclick={continue} onconfirm={continue}>{t("screen.loading.continue")}</button>{/if}
    </div>
  </div>
</div>

<style>
/* Art under a scrim fills the screen; the tip, the place and the bar stand
   at its foot. */
:root { position: relative; block-size: stretch; }
#Art, [part="loading.scrim"] { position: absolute; inset: 0; }
#Art { object-fit: cover; }
[part="loading.scrim"] { background: var(--scrim); }
p { margin: 0; }
#Foot { position: relative; display: flex; flex-direction: column; justify-content: end; gap: var(--space-md); block-size: 100%; padding: var(--space-3xl); }
[part="loading.frame"] { display: flex; flex-direction: column; gap: var(--space-sm); max-inline-size: var(--measure-xxl); }
/* Emphasised ink (--highlight), not --primary: that is a fill for the
   directed action, which a pack may make a deep colour no text reads in. */
#Eyebrow { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--highlight); }
#Text { font: var(--text-reading); letter-spacing: var(--type-reading-tracking); line-clamp: 4; }
[part="loading.title"] { margin: 0; font-size: var(--type-xl2-size); letter-spacing: var(--type-xl2-tracking); }
#Progress { display: flex; gap: var(--space-md); align-items: center; }
#Bar { flex: 1; block-size: var(--space-md); }
#Percent { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
@media (width < 600px), (height < 600px) {
  #Foot { padding: var(--space-xl); }
  [part="loading.title"] { font-size: var(--type-lg-size); }
}
</style>
