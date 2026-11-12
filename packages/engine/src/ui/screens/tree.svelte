<meta name="component" content="tree">
<meta name="prop:rows" content="list" data-doc="The visible rows in order, a keyed collection of { id, label, depth (0 at the top), open (true, false, or null for a leaf), selected }; the game flattens the tree as it opens and closes.">
<meta name="prop:label" content="string" data-doc="The tree's accessible name.">
<meta name="action:toggle" content="Open or close">
<meta name="action:choose" content="Select">
<meta name="fixture:default" content="{&quot;label&quot;:&quot;Scene&quot;,&quot;rows&quot;:[{&quot;id&quot;:&quot;w&quot;,&quot;label&quot;:&quot;World&quot;,&quot;depth&quot;:0,&quot;open&quot;:true,&quot;selected&quot;:false},{&quot;id&quot;:&quot;p&quot;,&quot;label&quot;:&quot;Player&quot;,&quot;depth&quot;:1,&quot;open&quot;:null,&quot;selected&quot;:true},{&quot;id&quot;:&quot;c&quot;,&quot;label&quot;:&quot;Camera&quot;,&quot;depth&quot;:1,&quot;open&quot;:false,&quot;selected&quot;:false}]}">

<list id="Tree" role="tree" aria-label={$label}>
  {#each $rows as r (r.id)}
    <div class="row" role="treeitem" aria-level={r.depth + 1} aria-selected={r.selected} aria-expanded={r.open} style:--depth={r.depth}>
      <button id="Twisty" class="twisty" variant="quiet" hidden={r.open == null} onclick={toggle} aria-label={r.label}><og-icon icon={r.open ? "chevronDown" : "chevronRight"}/></button>
      <button id="Item" class="item" variant={r.selected ? "primary" : "quiet"} onclick={choose}>{r.label}</button>
    </div>
  {/each}
</list>

<style>
#Tree { width: stretch; height: stretch; }
.row { display: flex; align-items: center; gap: 2px; margin-left: calc(var(--depth) * 16px); }
.twisty { width: 28px; }
.item { width: stretch; justify-content: start; }
</style>
