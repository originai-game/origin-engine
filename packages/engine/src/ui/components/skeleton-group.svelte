<meta name="component" content="SkeletonGroup">
<meta name="strings" content="./skeleton-group.en.ogt">
<meta name="prop:shape" content="string" data-doc="What it stands in for: lines (empty), avatar, block, row or card.">
<meta name="prop:items" content="list" data-doc="One entry per repeat (rows of a list), each with a stable id.">
<meta name="prop:name" content="string" data-doc="What a screen reader says while it loads; empty is Loading.">
<meta name="fixture:default" content='{"shape":"row","items":[{"id":"a"},{"id":"b"},{"id":"c"}],"name":""}'>

<div id="SkeletonGroup" role="progressbar" aria-busy="true" aria-label={name || t("component.skeleton-group.loading")} class:avatar={shape == 'avatar'} class:block={shape == 'block'} class:row={shape == 'row'} class:card={shape == 'card'}>
  {#each items as item (item.id)}
    <div class="item">
      <div class="art"></div>
      <div class="lines">
        <div class="piece first"></div>
        <div class="piece"></div>
        <div class="piece last"></div>
      </div>
    </div>
  {/each}
</div>

<style>
#SkeletonGroup { width: stretch; gap: 16px; animation: breathe 1600ms ease-in-out infinite; }
.item { width: stretch; gap: 8px; }
.art { display: none; background: var(--control-strong); border-radius: 8px; }
.lines { width: stretch; gap: 8px; }
.piece { width: stretch; height: 12px; border-radius: 6px; background: linear-gradient(90deg, #ffffff00 30%, #ffffff1f 50%, #ffffff00 70%), var(--control-strong); }
.piece.last { width: 62%; }
.avatar .art { display: block; width: 48px; height: 48px; border-radius: 24px; }
.avatar .lines { display: none; }
.block .art { display: block; width: stretch; height: 120px; }
.block .lines { display: none; }
.row .item { display: flex; align-items: center; gap: 16px; }
.row .art { display: block; width: 40px; height: 40px; border-radius: 20px; }
.row .piece.first { width: 60%; }
.row .piece.last { display: none; }
.card .art { display: block; width: stretch; height: 120px; }
.card .piece.first { width: 55%; height: 14px; }
@keyframes breathe { 50% { opacity: 0.6; } }
</style>
