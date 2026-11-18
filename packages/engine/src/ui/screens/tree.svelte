<meta name="component" content="tree">
<meta name="parts" content="tree.row">
<meta name="prop:rows" content="list" data-doc="The visible rows in order, a keyed collection of { id, label, depth (0 at the top), open (true, false, or null for a leaf), selected }; the game flattens the tree as it opens and closes.">
<meta name="prop:label" content="string" data-doc="The tree's accessible name.">
<meta name="action:toggle" content="Open or close">
<meta name="action:choose" content="Select">
<meta name="fixture:default" content="{&quot;label&quot;:&quot;Scene&quot;,&quot;rows&quot;:[{&quot;id&quot;:&quot;w&quot;,&quot;label&quot;:&quot;World&quot;,&quot;depth&quot;:0,&quot;open&quot;:true,&quot;selected&quot;:false},{&quot;id&quot;:&quot;p&quot;,&quot;label&quot;:&quot;Player&quot;,&quot;depth&quot;:1,&quot;open&quot;:null,&quot;selected&quot;:true},{&quot;id&quot;:&quot;c&quot;,&quot;label&quot;:&quot;Camera&quot;,&quot;depth&quot;:1,&quot;open&quot;:false,&quot;selected&quot;:false}]}">
<meta name="fixture:overflow" content="{&quot;label&quot;:&quot;Scene&quot;,&quot;rows&quot;:[{&quot;id&quot;:&quot;t0&quot;,&quot;label&quot;:&quot;World&quot;,&quot;depth&quot;:0,&quot;open&quot;:true,&quot;selected&quot;:false},{&quot;id&quot;:&quot;t1&quot;,&quot;label&quot;:&quot;Region of the Ninefold Gate&quot;,&quot;depth&quot;:1,&quot;open&quot;:true,&quot;selected&quot;:false},{&quot;id&quot;:&quot;t2&quot;,&quot;label&quot;:&quot;Lighthouse&quot;,&quot;depth&quot;:2,&quot;open&quot;:true,&quot;selected&quot;:false},{&quot;id&quot;:&quot;t3&quot;,&quot;label&quot;:&quot;Lantern_03 (the one on the north stair)&quot;,&quot;depth&quot;:3,&quot;open&quot;:null,&quot;selected&quot;:true},{&quot;id&quot;:&quot;t4&quot;,&quot;label&quot;:&quot;Player&quot;,&quot;depth&quot;:1,&quot;open&quot;:null,&quot;selected&quot;:false},{&quot;id&quot;:&quot;t5&quot;,&quot;label&quot;:&quot;Camera&quot;,&quot;depth&quot;:1,&quot;open&quot;:true,&quot;selected&quot;:false},{&quot;id&quot;:&quot;t6&quot;,&quot;label&quot;:&quot;Light&quot;,&quot;depth&quot;:2,&quot;open&quot;:true,&quot;selected&quot;:false},{&quot;id&quot;:&quot;t7&quot;,&quot;label&quot;:&quot;Particles&quot;,&quot;depth&quot;:3,&quot;open&quot;:null,&quot;selected&quot;:false},{&quot;id&quot;:&quot;t8&quot;,&quot;label&quot;:&quot;Audio&quot;,&quot;depth&quot;:1,&quot;open&quot;:false,&quot;selected&quot;:false}]}">

<!-- A HUD plate of rows, each indented by its depth with a faint guide
     down from its parent; the chosen row is washed in the accent. -->
<div id="Frame">
<list id="Tree" role="tree" aria-label={label}>
  {#each rows as r (r.id)}
    <div class="row" class:leaf={r.open == null} part="tree.row" role="treeitem" aria-level={r.depth + 1} aria-selected={r.selected} aria-expanded={r.open} style:--depth={r.depth}>
      <button id="Twisty" class="twisty" variant="quiet" hidden={r.open == null} onclick={toggle} aria-label={r.label}><og-icon icon={r.open ? "chevronDown" : "chevronRight"}/></button>
      <button id="Item" class="item" variant="quiet" onclick={choose}>{r.label}</button>
    </div>
  {/each}
</list>
</div>

<style>
#Frame { display: flex; flex-direction: column; width: stretch; height: stretch; padding-block: var(--space-xs); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(calc(2 * var(--hud-plate-cut)) 0, 100% 0, 100% calc(100% - 2 * var(--hud-plate-cut)), calc(100% - 2 * var(--hud-plate-cut)) 100%, 0 100%, 0 calc(2 * var(--hud-plate-cut))); color: var(--hud-ink); }
#Tree { width: stretch; height: stretch; --row-fill: none; --row-hover-fill: var(--data-row-hover-fill); --row-pressed-fill: var(--data-row-pressed-fill); --row-selected-fill: var(--data-row-selected-fill); --row-selected-edge: var(--data-row-selected-edge); --row-radius: 0px; }
/* A row is indented by its depth, the body type's size per level (an em,
   as desktop outlines step): the step grows with the player's text scale,
   so the levels stay apart when the names are large. The indent stops
   growing after four levels, so a deep row keeps room for its name. A
   chosen row is washed in the accent from its leading edge. */
.row { position: relative; display: flex; width: stretch; box-sizing: border-box; align-items: start; gap: var(--space-xxs); padding-inline: var(--space-sm) var(--space-md); padding-inline-start: calc(var(--space-sm) + min(var(--depth), 4) * var(--indent) + var(--twisty-room)); --indent: var(--type-body-size); --twisty-room: 0px; }
/* An indent guide: a faint line down the middle of its level's step. */
.row::before { content: ""; position: absolute; inset-block: 0; inset-inline-start: calc(var(--space-sm) + (min(var(--depth), 4) - 0.5) * var(--indent)); inline-size: var(--stroke-hairline); background: var(--data-guide); }
.row[aria-level="1"]::before { display: none; }
.twisty { flex: none; width: var(--control-sm); --button-quiet-ink: var(--hud-ink-dim); }
/* The twisty is its chevron alone: a pack's button padding would leave the
   chevron no width. */
.twisty::part(base) { padding-inline: 0; }
/* A leaf has no twisty and keeps its room, so its name lines up with its
   siblings' that open. */
.leaf { --twisty-room: calc(var(--control-sm) + var(--space-xxs)); }
/* A name longer than its row wraps inside it, at any word and inside a
   word with no break (an identifier): it is never cut by the screen. */
.item { flex: 1; min-width: 0; justify-content: start; text-align: start; white-space: normal; overflow-wrap: anywhere; --button-quiet-ink: var(--hud-ink); }
.row[aria-selected="true"] .item { --button-quiet-ink: var(--hud-accent); font-weight: var(--data-name-weight); }
/* A wrapped name keeps clear of its row's frame above and below. */
.item::part(base) { padding-block: var(--button-wrap-pad, var(--space-sm)); }
</style>
