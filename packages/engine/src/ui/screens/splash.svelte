<meta name="screen" content="splash" data-layer="modal" data-focus="" data-under="hide" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./splash.en.ogt">
<meta name="prop:marks" content="list" data-doc="The marks in order: { name, texture } (texture: an og.load.texture handle, 0 for the placeholder).">
<meta name="prop:mark" content="string" data-doc="The stable collection key selected now.">
<meta name="prop:progress" content="number" data-doc="Boot progress, 0 to 1 (below 0: no bar).">
<meta name="prop:status" content="string" data-doc="What is loading, small under the bar (empty: nothing).">
<meta name="action:confirm" content="Skip">
<meta name="fixture:default" content="{&quot;mark&quot;:&quot;item-0&quot;,&quot;marks&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Studio&quot;,&quot;texture&quot;:0},{&quot;id&quot;:&quot;item-1&quot;,&quot;name&quot;:&quot;Engine&quot;,&quot;texture&quot;:0}],&quot;progress&quot;:0.4,&quot;status&quot;:&quot;Loading shaders&quot;}">

<column id="Splash" interactive="true" onclick={skip} onconfirm={skip}>
  <spacer id="Node1"></spacer>
  {#each $marks as m (m.id)}
    <column id="Mark" hidden={!((m.id == $mark))}>
      <art id="Art" kind="logo" src={m.texture}></art>
      <text id="Name">{m.name}</text>
    </column>
  {/each}
  <column id="Custom">{@render custom?.()}</column>
  <spacer id="Node7"></spacer>
  <column id="Boot" hidden={!(($progress >= 0.0))}>
    <progress id="Bar" bind:value={$progress}></progress>
    <text id="Status">{$status}</text>
  </column>
</column>

<style>
#Splash { width: stretch; height: stretch; padding: 48px; gap: 24px; justify-content: center; align-items: center; background: var(--backdrop); }
@media phonePortrait phoneLandscape {
#Splash { padding: 24px; }
}
#Mark { gap: 12px; align-items: center; -og-enter: fade; -og-exit: fade; }
#Art { width: 360px; height: 160px; }
@media phonePortrait phoneLandscape {
#Art { width: 260px; height: 116px; }
}
#Name { font: var(--text-caption); color: var(--ink-muted); }
#Boot { width: stretch; max-width: 360px; gap: 8px; align-items: center; }
#Bar { width: stretch; }
#Status { font: var(--text-caption); color: var(--ink-muted); line-clamp: 1; text-overflow: ellipsis; }
</style>
