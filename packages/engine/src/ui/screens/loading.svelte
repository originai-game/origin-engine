<meta name="screen" content="loading" data-layer="modal" data-focus="" data-under="hide" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./loading.en.ogt">
<meta name="parts" content="loading.scrim loading.frame loading.title loading.eyebrow loading.label loading.next loading.bar loading.fill loading.figure loading.status">
<meta name="prop:keyArt" content="number" data-doc="The key art's texture (og.load.texture; 0: the style's placeholder).">
<meta name="prop:place" content="string" data-doc="Where the player is going.">
<meta name="prop:tip" content="string" data-doc="A tip or a line of lore.">
<meta name="prop:progress" content="number" data-doc="0 to 1.">
<meta name="prop:ready" content="bool" data-doc="Loading is done.">
<meta name="prop:waits" content="bool" data-doc="The game waits for a press once ready.">
<meta name="prop:status" content="string" data-default="" data-doc="What the wait is for, beside the figure while it is not ready (Waiting for the server); empty: none.">
<meta name="action:confirm" content="Continue">
<meta name="fixture:default" content="{&quot;keyArt&quot;:0,&quot;place&quot;:&quot;The Sunken Archive&quot;,&quot;progress&quot;:0.62,&quot;ready&quot;:false,&quot;tip&quot;:&quot;Dodging through an attack at the last moment slows time for a breath.&quot;,&quot;waits&quot;:true}">

<!--
  The key art fills the screen. At its foot, over a shade that grows with
  the words: where the player is going (an eyebrow and the place), the tip
  on a cut smoked-glass plate, and the progress as the HUD's leaning bar
  with a spinner and the figure; once ready, Continue (or nothing, when the
  game does not wait). A press anywhere asks for the next tip.
-->
<div id="Loading" interactive="true" onclick={nextTip}>
  {#if keyArt}<art id="Art" kind="tutorial" src={keyArt}></art>{/if}
  {#if !keyArt}<div id="Placeholder"></div>{/if}
  <div id="Scrim" part="loading.scrim"></div>
  <div id="Foot" data-layout="stack">
    <div id="Top">
      <div id="Where" data-layout="stack" data-gap="hair">
        <div id="Eyebrow" part="loading.eyebrow" data-motion="enter"><span>{t("screen.loading.entering")}</span></div>
        <h2 id="Place" part="loading.title" data-motion="enter">{place}</h2>
      </div>
      <section id="Tip" part="loading.frame" class="bare" data-layout="stack" data-gap="tight" data-motion="enter">
        <div id="TipHead" data-layout="row">
          <p id="Eyebrow2" part="loading.label">{t("screen.loading.tip")}</p>
          <button id="Next" part="loading.next" variant="quiet" onclick={nextTip}><span class="key">{t("screen.loading.nextKey")}</span><span>{t("screen.loading.next")}</span></button>
        </div>
        <p id="Text">{tip}</p>
        <div id="Lore">{@render lore?.()}</div>
      </section>
    </div>
    <div id="Progress">
      <div id="Bar" part="loading.bar" data-layout="overlay" role="progressbar" aria-label={place} aria-valuemin="0" aria-valuemax="1" aria-valuenow={progress} data-ready={ready} style:--value={progress <= 0 ? 0 : progress >= 1 ? 1 : progress}>
        <div id="Fill" part="loading.fill" data-anchor="top-start"></div>
      </div>
      <div id="Readout" data-layout="row">
        {#if !ready}<Spinner size={22} name={t("screen.loading.busy")} />{/if}
        {#if !ready && status}<p id="Status" part="loading.status">{status}</p>{/if}
        {#if !(ready && waits)}<p id="Percent" part="loading.figure" data-ready={ready}>{t("screen.loading.Percent", { progress: formatPercent(progress) })}</p>{/if}
        {#if ready && waits}<button id="Continue" class="go" variant="primary" autofocus="true" data-motion="pop" onclick={continue} onconfirm={continue}><span class="key">{t("screen.loading.continueKey")}</span><span class="pointer">{t("screen.loading.continue")}</span><span class="touch">{t("screen.loading.tapContinue")}</span></button>{/if}
      </div>
    </div>
  </div>
</div>

<style>
:root { position: relative; block-size: stretch; overflow: hidden; color: var(--hud-ink); container-type: size; }
#Art, #Placeholder, [part="loading.scrim"] { position: absolute; inset: 0; }
#Art { object-fit: cover; }
#Placeholder { background: var(--loading-placeholder); }
[part="loading.scrim"] { background: var(--loading-side-shade); }
p { margin: 0; }
/* The foot stands at the bottom; its shade starts above it and grows with
   it (the title is held to a share of the screen, so it stays on it). */
#Foot { position: absolute; inset: auto 0 0 0; isolation: isolate; gap: calc(20 * var(--font-step-px)); padding: calc(56 * var(--font-step-px)) calc(64 * var(--font-step-px)) calc(48 * var(--font-step-px)); padding-inline: calc(64 * var(--font-step-px) + env(-og-hud-inset-left)) calc(64 * var(--font-step-px) + env(-og-hud-inset-right)); padding-block-end: calc(48 * var(--font-step-px) + env(-og-hud-inset-bottom)); }
/* The shade starts above the words and stays below the safe area's top. */
#Foot::before { content: ""; position: absolute; z-index: -1; inset: auto 0 0 0; block-size: calc(100% + 120 * var(--font-step-px)); max-block-size: calc(100cqb - env(-og-hud-inset-top)); background: var(--loading-shade); }
#Top { display: flex; flex-direction: column; gap: calc(20 * var(--font-step-px)); }
#Where { max-inline-size: 60%; }
[part="loading.eyebrow"] { display: flex; align-items: center; gap: calc(10 * var(--font-step-px)); font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-accent); }
[part="loading.eyebrow"]::before { content: ""; flex: none; inline-size: calc(28 * var(--font-step-px)); block-size: calc(2 * var(--font-step-px)); background: var(--hud-accent); box-shadow: var(--hud-pip-glow); }
/* The place is held to a share of the screen, so a long name in a pack's
   wide display face keeps whole words and stays on the screen. */
[part="loading.title"] { margin: 0; font-size: min(calc(54 * var(--font-step-px)), 7cqb, 6cqi); line-height: 1.04; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); color: var(--hud-ink); overflow-wrap: break-word; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
/* The tip: a cut smoked-glass plate, never clamped. */
[part="loading.frame"] { max-inline-size: calc(760 * var(--font-step-px)); padding: calc(16 * var(--font-step-px)) calc(22 * var(--font-step-px)) calc(18 * var(--font-step-px)); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: calc(2 * var(--hud-plate-cut)) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
#TipHead { align-items: center; column-gap: var(--space-sm); }
[part="loading.label"] { flex: none; padding: calc(2 * var(--font-step-px)) calc(8 * var(--font-step-px)); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-accent); background: var(--hud-tag-fill); box-shadow: var(--hud-plate-lit-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
[part="loading.next"] { flex: 0 1 auto; min-inline-size: 0; margin-inline-start: auto; --button-quiet-ink: var(--hud-ink-dim); }
[part="loading.next"]::part(base) { column-gap: var(--space-xs); padding-inline: var(--space-xs); font-size: var(--hud-tag-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); }
.key { flex: none; min-inline-size: calc(22 * var(--font-step-px)); padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.6; font-weight: var(--hud-figure-weight); text-align: center; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); }
#Text { font: var(--text-reading); letter-spacing: var(--type-reading-tracking); color: var(--hud-ink); overflow-wrap: anywhere; }
#Lore { font: var(--text-caption); font-style: italic; color: var(--hud-ink-dim); }
/* The progress: the HUD's leaning bar with a white lead edge, then the
   spinner, what it waits for and the figure (or Continue). */
#Progress { display: flex; align-items: center; gap: calc(18 * var(--font-step-px)); }
[part="loading.bar"] { flex: 1 1 auto; min-inline-size: 0; block-size: calc(12 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); --layout-anchor-inset: 0; }
[part="loading.fill"] { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--loading-fill); box-shadow: var(--loading-fill-glow), inset calc(-3 * var(--font-step-px)) 0 0 var(--loading-lead); transition: inline-size var(--motion-duration-slow) var(--motion-easing-exit); }
[data-ready=true] [part="loading.fill"] { box-shadow: var(--loading-fill-glow); }
#Readout { flex: none; align-items: center; column-gap: calc(12 * var(--font-step-px)); min-inline-size: 0; }
[part="loading.status"] { min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
[part="loading.figure"] { min-inline-size: calc(72 * var(--font-step-px)); font-size: calc(40 * var(--font-step-px)); line-height: 1; text-align: end; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
[part="loading.figure"][data-ready=true] { color: var(--hud-accent); }
#Continue { --button-transform: var(--hud-label-transform); --button-tracking: var(--hud-label-tracking);   }
#Continue::part(base) { column-gap: var(--space-sm); min-block-size: var(--touch-target); padding-inline: calc(22 * var(--font-step-px)); font-weight: var(--hud-figure-weight); white-space: normal; }
#Continue span { text-transform: var(--hud-label-transform); letter-spacing: var(--hud-label-tracking); }
#Continue .key { color: var(--hud-ink); background: var(--hud-key-ink); box-shadow: none; }
.touch { display: none; }
/* A touch screen has no keys: no key caps, Continue says to tap, and Next
   tip is a finger's target. */
@media (pointer: coarse) {
  .key, .pointer { display: none; }
  .touch { display: inline; }
  [part="loading.next"] { min-block-size: var(--touch-target); }
}
/* A phone: the words take the width, the figure (or Continue) sits on a
   row above a full-width bar. */
@media (width < 600px) {
  #Foot { gap: calc(14 * var(--font-step-px)); padding-block-start: calc(20 * var(--font-step-px)); padding-inline: calc(20 * var(--font-step-px) + env(-og-hud-inset-left)) calc(20 * var(--font-step-px) + env(-og-hud-inset-right)); padding-block-end: calc(24 * var(--font-step-px) + env(-og-hud-inset-bottom)); }
  #Top { gap: calc(14 * var(--font-step-px)); }
  #Where { max-inline-size: 100%; }
  [part="loading.title"] { font-size: min(calc(32 * var(--font-step-px)), 4cqb, 8cqi); }
  [part="loading.frame"] { padding: calc(12 * var(--font-step-px)) calc(14 * var(--font-step-px)); }
  #Progress { flex-direction: column-reverse; align-items: stretch; gap: calc(12 * var(--font-step-px)); }
  [part="loading.bar"] { flex: none; inline-size: stretch; }
  #Readout { flex-wrap: wrap; }
  #Continue { flex: 1 1 auto; }
  [part="loading.figure"] { min-inline-size: 0; font-size: calc(28 * var(--font-step-px)); margin-inline-start: auto; }
}
/* A short landscape screen: the place and the tip side by side, so the art
   keeps its upper half. */
@media (height < 500px) {
  #Foot { gap: calc(10 * var(--font-step-px)); padding-block: calc(16 * var(--font-step-px)) calc(16 * var(--font-step-px) + env(-og-hud-inset-bottom)); padding-inline: calc(24 * var(--font-step-px) + env(-og-hud-inset-left)) calc(24 * var(--font-step-px) + env(-og-hud-inset-right)); }
  #Top { flex-direction: row; align-items: end; gap: calc(16 * var(--font-step-px)); }
  #Where { flex: 1 1 0; max-inline-size: 45%; min-inline-size: 0; }
  [part="loading.frame"] { flex: 1.3 1 0; min-inline-size: 0; padding: calc(10 * var(--font-step-px)) calc(14 * var(--font-step-px)); }
  [part="loading.title"] { font-size: min(calc(26 * var(--font-step-px)), 9cqb, 4cqi); }
  #Lore { display: none; }
  [part="loading.figure"] { font-size: calc(24 * var(--font-step-px)); }
}
</style>
