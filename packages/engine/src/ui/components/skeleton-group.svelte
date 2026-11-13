<meta name="component" content="SkeletonGroup">
<meta name="strings" content="./skeleton-group.en.ogt">
<meta name="parts" content="SkeletonGroup.art SkeletonGroup.line">
<meta name="prop:shape" content="string" data-doc="What it stands in for: lines (empty), avatar, block, row or card.">
<meta name="prop:items" content="list" data-doc="One entry per repeat (rows of a list), each with a stable id.">
<meta name="prop:name" content="string" data-doc="What a screen reader says while it loads; empty is Loading.">
<meta name="fixture:default" content='{"shape":"row","items":[{"id":"a"},{"id":"b"},{"id":"c"}],"name":""}'>

<div id="SkeletonGroup" role="progressbar" aria-busy="true" aria-label={name || t("component.skeleton-group.loading")} class:avatar={shape == 'avatar'} class:block={shape == 'block'} class:row={shape == 'row'} class:card={shape == 'card'}>
  {#each items as item (item.id)}
    <div class="item">
      <div class="art" part="SkeletonGroup.art"></div>
      <div class="lines">
        <div class="piece first" part="SkeletonGroup.line"></div>
        <div class="piece" part="SkeletonGroup.line"></div>
        <div class="piece last" part="SkeletonGroup.line"></div>
      </div>
    </div>
  {/each}
</div>

<style>
#SkeletonGroup { width: stretch; gap: var(--space-lg); animation-name: breathe; animation-duration: calc(var(--motion-duration-slow) * 4); animation-timing-function: var(--motion-easing-in-out); animation-iteration-count: infinite; }
.item { width: stretch; gap: var(--space-sm); }
.art { display: none; background: var(--control-strong); border-radius: var(--radius-block); }
.lines { width: stretch; gap: var(--space-sm); }
.piece { width: stretch; height: var(--space-md); border-radius: var(--radius-capsule); background: linear-gradient(90deg, transparent 30%, var(--sheen) 50%, transparent 70%), var(--control-strong); }
.piece.last { width: 62%; }
.avatar .art { display: block; width: var(--size-lg); height: var(--size-lg); border-radius: var(--radius-capsule); }
.avatar .lines { display: none; }
.block .art { display: block; width: stretch; height: var(--size-3xl); }
.block .lines { display: none; }
.row .item { display: flex; align-items: center; gap: var(--space-lg); }
.row .art { display: block; width: var(--size-touch); height: var(--size-touch); border-radius: var(--radius-capsule); }
.row .piece.first { width: 60%; }
.row .piece.last { display: none; }
.card .art { display: block; width: stretch; height: var(--size-3xl); }
.card .piece.first { width: 55%; height: var(--space-lg); }
@keyframes breathe { 50% { opacity: 0.6; } }
</style>
