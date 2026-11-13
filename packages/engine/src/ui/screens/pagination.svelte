<meta name="component" content="pagination">
<meta name="parts" content="pagination.frame pagination.gap">
<meta name="prop:pages" content="list" data-doc="The page buttons, a keyed collection of { id, label, current }; a gap is { id, label: &quot;…&quot;, gap: true }.">
<meta name="prop:label" content="string" data-doc="The navigation's accessible name (Pages).">
<meta name="prop:atStart" content="bool" data-doc="The first page is shown: Previous takes no press.">
<meta name="prop:atEnd" content="bool" data-doc="The last page is shown: Next takes no press.">
<meta name="action:previous" content="Previous page">
<meta name="action:next" content="Next page">
<meta name="action:page" content="Go to page">
<meta name="fixture:default" content="{&quot;label&quot;:&quot;Pages&quot;,&quot;atStart&quot;:false,&quot;atEnd&quot;:false,&quot;pages&quot;:[{&quot;id&quot;:&quot;1&quot;,&quot;label&quot;:&quot;1&quot;,&quot;current&quot;:false},{&quot;id&quot;:&quot;2&quot;,&quot;label&quot;:&quot;2&quot;,&quot;current&quot;:true},{&quot;id&quot;:&quot;3&quot;,&quot;label&quot;:&quot;3&quot;,&quot;current&quot;:false},{&quot;id&quot;:&quot;gap&quot;,&quot;label&quot;:&quot;…&quot;,&quot;gap&quot;:true},{&quot;id&quot;:&quot;9&quot;,&quot;label&quot;:&quot;9&quot;,&quot;current&quot;:false}]}">

<nav id="Pagination" part="pagination.frame" class="bare" aria-label={$label}>
  <button id="Prev" variant="quiet" disabled={$atStart} onclick={previous} aria-label={$label}><og-icon icon="chevronLeft"/></button>
  <ol id="Pages">
    {#each $pages as p (p.id)}
      <li class="page">
        {#if p.gap}<span class="gap" part="pagination.gap">{p.label}</span>{/if}
        {#if !p.gap}<button id="Page" variant={p.current ? "primary" : "quiet"} aria-current={p.current} onclick={page}>{p.label}</button>{/if}
      </li>
    {/each}
  </ol>
  <button id="Next" variant="quiet" disabled={$atEnd} onclick={next} aria-label={$label}><og-icon icon="chevronRight"/></button>
</nav>

<style>
#Pagination { display: flex; align-items: center; gap: var(--space-xs); }
#Pages { display: flex; list-style: none; padding: 0; margin: 0; gap: var(--space-xs); }
.gap { color: var(--ink-muted); padding: 0 var(--space-xs); }
</style>
