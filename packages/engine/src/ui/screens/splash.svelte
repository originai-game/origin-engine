<meta name="screen" content="splash" data-layer="modal" data-focus="" data-under="hide" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./splash.en.ogt">
<meta name="parts" content="splash.frame splash.mark splash.dots splash.dot splash.boot splash.track splash.fill splash.skip">
<meta name="prop:marks" content="list" data-doc="The marks in order: { name, texture } (texture: an og.load.texture handle; 0 sets the name as a wordmark).">
<meta name="prop:mark" content="string" data-doc="The stable collection key selected now.">
<meta name="prop:progress" content="number" data-doc="Boot progress, 0 to 1 (below 0: no bar).">
<meta name="prop:status" content="string" data-doc="What is loading, small beside the bar's figure (empty: nothing).">
<meta name="action:confirm" content="Skip">
<meta name="fixture:default" content="{&quot;mark&quot;:&quot;item-0&quot;,&quot;marks&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Studio&quot;,&quot;texture&quot;:0},{&quot;id&quot;:&quot;item-1&quot;,&quot;name&quot;:&quot;Engine&quot;,&quot;texture&quot;:0}],&quot;progress&quot;:0.4,&quot;status&quot;:&quot;Loading shaders&quot;}">

<div id="Splash" part="splash.frame" class="bare" interactive="true" onclick={skip} onconfirm={skip}>
  <div id="Stage">
    {#each marks as m (m.id)}
      <div id="Mark" part="splash.mark" hidden={m.id != mark} aria-label={m.name}>
        {#if m.texture != 0}
          <art id="Art" kind="logo" src={m.texture}></art>
        {:else}
          <p id="Word" class="word">{m.name}</p>
        {/if}
      </div>
    {/each}
    {#if marks.length > 1}
      <div id="Dots" part="splash.dots">
        {#each marks as m (m.id)}<span id="Dot" part="splash.dot" class:on={m.id == mark}></span>{/each}
      </div>
    {/if}
  </div>
  <div id="Custom">{@render custom?.()}</div>
  {#if progress >= 0}
    <div id="Boot" part="splash.boot" role="progressbar" aria-valuemin="0" aria-valuemax="1" aria-valuenow={progress > 1 ? 1 : progress} aria-label={status}>
      <div id="Row">
        <p id="Status">{status}</p>
        <og-counter id="Percent" value={progress > 1 ? 1 : progress} format="percent" decimals="0"></og-counter>
      </div>
      <div id="Track" part="splash.track" style:--value={progress > 1 ? 1 : progress}><div id="Fill" part="splash.fill"></div></div>
    </div>
  {/if}
  <div id="Skip" part="splash.skip"><span class="key">{t("screen.splash.skipKey")}</span><span class="press">{t("screen.splash.skip")}</span><span class="tap">{t("screen.splash.tap")}</span></div>
</div>

<style>
/* The marks centred on a dark ground, a dot for each under them; the boot
   line at the foot, a lean bar under its status and figure; how to skip
   in the corner. */
:root { position: relative; display: flex; flex-direction: column; align-items: center; row-gap: var(--space-xl); block-size: stretch; padding: var(--space-3xl); background: var(--front-ground); color: var(--front-ink); }
p { margin: 0; }
#Stage { display: flex; flex-direction: column; align-items: center; row-gap: var(--space-xl); inline-size: 100%; margin-block-start: auto; container-type: inline-size; }
#Custom { margin-block-end: auto; }
/* A mark fades in and out as the splash moves to the next. */
[part="splash.mark"] { display: flex; flex-direction: column; align-items: center; row-gap: var(--space-md); inline-size: 100%; transition: opacity var(--motion-enter), display var(--motion-enter) allow-discrete; }
[part="splash.mark"][hidden] { display: none; opacity: 0; transition: opacity var(--motion-exit), display var(--motion-exit) allow-discrete; }
@starting-style { [part="splash.mark"] { opacity: 0; } }
#Art { inline-size: min(100%, calc(520 * var(--font-step-px))); aspect-ratio: 2; block-size: auto; object-fit: contain; }
/* A mark without art is its name, set as a wordmark (whole words, sized to
   a share of the stage). */
.word { max-inline-size: 100%; font-size: min(calc(48 * var(--font-step-px)), 9cqi); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); font-stretch: var(--front-stretch); letter-spacing: var(--front-heading-tracking); text-transform: uppercase; text-align: center; overflow-wrap: normal; hyphens: none; text-shadow: var(--front-shadow); }
[part="splash.dots"] { display: flex; column-gap: var(--space-xs); }
[part="splash.dot"] { inline-size: calc(18 * var(--font-step-px)); block-size: var(--stroke-thin); background: var(--front-dot); transition: background-color var(--motion-ui-duration) var(--motion-ui-easing); }
[part="splash.dot"].on { background: var(--front-dot-on); }
[part="splash.boot"] { display: flex; flex-direction: column; row-gap: var(--space-sm); inline-size: 100%; max-inline-size: calc(420 * var(--font-step-px)); }
#Row { display: flex; align-items: baseline; column-gap: var(--space-md); }
#Status { flex: 1 1 auto; min-inline-size: 0; font-size: calc(12 * var(--font-step-px)); font-weight: var(--font-weight-semibold); letter-spacing: var(--hud-label-tracking); text-transform: uppercase; color: var(--front-ink-faint); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
#Percent { flex: none; font-size: calc(13 * var(--font-step-px)); font-weight: var(--font-weight-bold); font-variant-numeric: tabular-nums; color: var(--front-ink-dim); }
[part="splash.track"] { block-size: calc(4 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--front-boot-well); }
[part="splash.fill"] { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--front-boot-fill); box-shadow: var(--front-boot-glow); transition: inline-size var(--motion-ui-duration) var(--motion-ui-easing); }
[part="splash.skip"] { position: absolute; inset-inline-end: var(--space-xl); inset-block-end: var(--space-lg); display: flex; align-items: center; column-gap: var(--space-sm); font-size: calc(13 * var(--font-step-px)); color: var(--front-ink-dim); }
.key { padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.6; font-weight: var(--font-weight-bold); color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); }
.tap { display: none; }
/* On a touch screen the skip names the tap and shows no key. */
@media (pointer: coarse) {
  .key, .press { display: none; }
  .tap { display: inline; }
}
@media (width < 600px), (height < 600px) {
  :root { padding: var(--space-xxl) var(--space-xl) calc(var(--space-3xl) + var(--space-md)); }
  #Art { inline-size: 80%; }
  [part="splash.skip"] { inset-inline: var(--space-lg); inset-block-end: var(--space-lg); justify-content: center; }
}
</style>
