<meta name="screen" content="splash" data-layer="modal" data-focus="" data-under="hide" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./splash.en.ogt">
<meta name="parts" content="splash.frame splash.mark">
<meta name="prop:marks" content="list" data-doc="The marks in order: { name, texture } (texture: an og.load.texture handle, 0 for the placeholder).">
<meta name="prop:mark" content="string" data-doc="The stable collection key selected now.">
<meta name="prop:progress" content="number" data-doc="Boot progress, 0 to 1 (below 0: no bar).">
<meta name="prop:status" content="string" data-doc="What is loading, small under the bar (empty: nothing).">
<meta name="action:confirm" content="Skip">
<meta name="fixture:default" content="{&quot;mark&quot;:&quot;item-0&quot;,&quot;marks&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Studio&quot;,&quot;texture&quot;:0},{&quot;id&quot;:&quot;item-1&quot;,&quot;name&quot;:&quot;Engine&quot;,&quot;texture&quot;:0}],&quot;progress&quot;:0.4,&quot;status&quot;:&quot;Loading shaders&quot;}">

<div id="Splash" part="splash.frame" class="bare" interactive="true" onclick={skip} onconfirm={skip}>
  {#each marks as m (m.id)}
    <div id="Mark" part="splash.mark" hidden={m.id != mark}>
      <art id="Art" kind="logo" src={m.texture}></art>
      <p id="Name">{m.name}</p>
    </div>
  {/each}
  <div id="Custom">{@render custom?.()}</div>
  {#if progress >= 0}
    <div id="Boot">
      <progress id="Bar" bind:value={progress}></progress>
      <p id="Status">{status}</p>
    </div>
  {/if}
</div>

<style>
/* The marks centred above the boot line, which sits at the foot. */
:root { display: flex; flex-direction: column; align-items: center; gap: var(--space-xl); block-size: stretch; padding: var(--space-3xl); background: var(--backdrop); }
p { margin: 0; }
:root > :first-child { margin-block-start: auto; }
#Custom { margin-block-end: auto; }
/* A mark fades in and out as the splash moves to the next. */
[part="splash.mark"] { display: flex; flex-direction: column; align-items: center; gap: var(--space-md); transition: opacity var(--motion-enter), display var(--motion-enter) allow-discrete; }
[part="splash.mark"][hidden] { display: none; opacity: 0; transition: opacity var(--motion-exit), display var(--motion-exit) allow-discrete; }
@starting-style { [part="splash.mark"] { opacity: 0; } }
#Art { inline-size: var(--measure-md); block-size: var(--size-4xl); }
#Name, #Status { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#Boot { display: flex; flex-direction: column; align-items: center; gap: var(--space-sm); inline-size: 100%; max-inline-size: var(--measure-md); }
#Status { line-clamp: 1; text-overflow: ellipsis; }
@media (width < 600px), (height < 600px) {
  :root { padding: var(--space-xl); }
  #Art { inline-size: var(--size-7xl); block-size: var(--size-3xl); }
}
</style>
