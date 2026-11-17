<meta name="component" content="SkeletonGroup">
<meta name="strings" content="./skeleton-group.en.ogt">
<meta name="parts" content="SkeletonGroup.art SkeletonGroup.line SkeletonGroup.skeleton SkeletonGroup.sheen">
<meta name="prop:shape" content="string" data-doc="What it stands in for: lines (empty), avatar, block, row or card.">
<meta name="prop:items" content="list" data-doc="One entry per repeat (rows of a list), each with a stable id.">
<meta name="prop:name" content="string" data-doc="What a screen reader says while it loads; empty is Loading.">
<meta name="fixture:default" content='{"shape":"row","items":[{"id":"a"},{"id":"b"},{"id":"c"}],"name":""}'>

<div id="SkeletonGroup" data-layout="stack" data-gap="loose" role="progressbar" aria-busy="true" aria-label={name || t("component.skeleton-group.loading")} class:avatar={shape == 'avatar'} class:block={shape == 'block'} class:row={shape == 'row'} class:card={shape == 'card'}>
  {#each items as item (item.id)}
    <div class="item" data-layout={shape == 'row' ? 'row' : 'stack'} data-gap="tight">
      {#if shape == 'avatar' || shape == 'block' || shape == 'row' || shape == 'card'}<div class="art" part="SkeletonGroup.art SkeletonGroup.skeleton"><div part="SkeletonGroup.sheen" data-motion="shimmer"></div></div>{/if}
      {#if shape != 'avatar' && shape != 'block'}<div class="lines" data-layout="stack" data-gap="tight" data-fill>
        <div class="piece first" part="SkeletonGroup.line SkeletonGroup.skeleton"><div part="SkeletonGroup.sheen" data-motion="shimmer"></div></div>
        <div class="piece" part="SkeletonGroup.line SkeletonGroup.skeleton"><div part="SkeletonGroup.sheen" data-motion="shimmer"></div></div>
        {#if shape != 'row'}<div class="piece last" part="SkeletonGroup.line SkeletonGroup.skeleton"><div part="SkeletonGroup.sheen" data-motion="shimmer"></div></div>{/if}
      </div>{/if}
    </div>
  {/each}
</div>

<style>
:host { inline-size: stretch; }
.piece { width: stretch; height: var(--space-md); --skeleton-radius: var(--radius-capsule); }
.piece.last { width: 62%; }
.avatar .art { width: var(--size-lg); height: var(--size-lg); --skeleton-radius: var(--radius-capsule); }
.block .art { height: var(--size-3xl); }
.row .item { --layout-gap-tight: var(--layout-gap-loose); }
.row .art { width: var(--size-touch); height: var(--size-touch); --skeleton-radius: var(--radius-capsule); }
.row .piece.first { width: 60%; }
.card .art { height: var(--size-3xl); }
.card .piece.first { width: 55%; height: var(--space-lg); }
</style>
