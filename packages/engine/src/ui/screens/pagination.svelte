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

<!-- A row of cut keys, the current page lit. -->
<nav id="Pagination" part="pagination.frame" class="bare" data-layout="row" aria-label={label}>
  <button id="Prev" variant="quiet" disabled={atStart} onclick={previous} aria-label={label}><og-icon icon="chevronLeft"/></button>
  <ol id="Pages" data-layout="cluster">
    {#each pages as p (p.id)}
      <li class="page">
        {#if p.gap}<span class="gap" part="pagination.gap">{p.label}</span>{/if}
        {#if !p.gap}<button id="Page" variant={p.current ? "primary" : "quiet"} aria-current={p.current} onclick={page}>{p.label}</button>{/if}
      </li>
    {/each}
  </ol>
  <button id="Next" variant="quiet" disabled={atEnd} onclick={next} aria-label={label}><og-icon icon="chevronRight"/></button>
</nav>

<style>
/* Cut keys: the pages in condensed tabular figures, the current one lit in
   the accent. The arrows keep their ends; the pages between them wrap onto
   more lines where the row is too narrow for them all. */
#Pagination { max-inline-size: 100%; gap: var(--space-xs); align-items: center; }
#Prev, #Next { flex: none; min-inline-size: var(--touch-target); min-block-size: var(--touch-target); --button-radius: var(--hud-tag-cut); --button-corner-shape: bevel; --button-quiet-ink: var(--hud-ink); }
#Prev::part(base), #Next::part(base) { padding-inline: 0; }
#Pages { flex: 0 1 auto; min-inline-size: 0; justify-content: center; list-style: none; padding: 0; margin: 0; gap: var(--space-xs); }
#Page { min-inline-size: var(--touch-target); min-block-size: var(--touch-target); --button-radius: var(--hud-tag-cut); --button-corner-shape: bevel; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); --button-quiet-ink: var(--hud-ink); }
#Page[variant="quiet"] { background: var(--data-well); box-shadow: var(--data-well-rim); }
#Page[variant="primary"] { --button-primary-fill: var(--hud-key-lit-fill); --button-primary-hover-fill: var(--hud-key-lit-fill); --button-primary-pressed-fill: var(--hud-key-lit-fill); --button-primary-ink: var(--hud-key-ink); }
#Page[variant="primary"]::part(base) { color: var(--hud-key-ink); }
.gap { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--data-head-ink); padding: 0 var(--space-xs); }
</style>
