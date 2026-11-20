<meta name="component" content="card">
<meta name="parts" content="card.frame card.image card.title card.body">
<meta name="prop:title" content="string" data-doc="Its heading.">
<meta name="prop:body" content="string" data-doc="A line or two under the heading; empty: none.">
<meta name="prop:image" content="string" data-doc="A picture the game registered (UiDesc.images) across its top; empty: none.">
<meta name="prop:badge" content="string" data-doc="A badge at its top end (New, -20%); empty: none.">
<meta name="action:open" content="Open">
<meta name="fixture:default" content="{&quot;title&quot;:&quot;Ember Blade&quot;,&quot;body&quot;:&quot;Burns on hit. Two-handed.&quot;,&quot;image&quot;:&quot;&quot;,&quot;badge&quot;:&quot;New&quot;}">

<article id="Card" part="card.frame" class="bare">
  <button id="Hit" class="hit" onclick={open} aria-label={title}>
    <div id="Face">
    <img id="Image" part="card.image" src={image} alt="" hidden={image == ""}>
    <div id="Head">
      <h3 id="Title" part="card.title">{title}</h3>
      <badge id="Badge" hidden={badge == ""}>{badge}</badge>
    </div>
    <p id="Body" part="card.body" hidden={body == ""}>{body}</p>
    </div>
  </button>
  <footer id="Actions" data-layout="cluster">{@render actions?.()}</footer>
</article>

<style>
#Card { display: flex; flex-direction: column; width: var(--measure-sm); max-inline-size: 100%; gap: var(--space-sm); }
/* The card is a HUD plate the player presses; its words wrap, never cut. */
.hit { padding: var(--space-md); width: stretch; white-space: normal; text-align: start; background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: calc(2 * var(--hud-plate-cut)) 0; corner-shape: var(--cut-shape); clip-path: border-box; color: var(--hud-ink); --button-radius: 0px; }
#Face { display: flex; flex-direction: column; gap: var(--space-sm); width: stretch; min-width: 0; }
#Image { width: stretch; aspect-ratio: 1.778; border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
#Head { display: flex; flex-wrap: wrap; align-items: start; gap: var(--space-xs) var(--space-sm); }
#Title { flex: 1 1 calc(var(--measure-xs) * 0.6); min-width: 0; margin: 0; font-size: var(--hud-size-sm); font-weight: var(--data-title-weight); color: var(--hud-ink); text-align: start; overflow-wrap: break-word; }
#Badge { flex: none; font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); --badge-fill: var(--hud-tag-fill); --badge-ink: var(--hud-gold); --badge-shadow: inset 0 0 0 var(--stroke-hairline) rgb(from var(--hud-gold) r g b / 0.45); --badge-radius: 0px; }
#Body { margin: 0; color: var(--hud-ink-dim); text-align: start; overflow-wrap: anywhere; }
#Actions { justify-content: end; gap: var(--space-sm); }
</style>
