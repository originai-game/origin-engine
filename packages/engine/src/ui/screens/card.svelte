<meta name="component" content="card">
<meta name="parts" content="card.frame card.image card.title card.body">
<meta name="prop:title" content="string" data-doc="Its heading.">
<meta name="prop:body" content="string" data-doc="A line or two under the heading; empty: none.">
<meta name="prop:image" content="string" data-doc="A picture the game registered (UiDesc.images) across its top; empty: none.">
<meta name="prop:badge" content="string" data-doc="A badge at its top end (New, -20%); empty: none.">
<meta name="action:open" content="Open">
<meta name="fixture:default" content="{&quot;title&quot;:&quot;Ember Blade&quot;,&quot;body&quot;:&quot;Burns on hit. Two-handed.&quot;,&quot;image&quot;:&quot;&quot;,&quot;badge&quot;:&quot;New&quot;}">

<article id="Card" part="card.frame" class="bare">
  <button id="Hit" class="hit" onclick={open} aria-label={$title}>
    <div id="Face">
    <img id="Image" part="card.image" src={$image} alt="" hidden={$image == ""}>
    <div id="Head">
      <h3 id="Title" part="card.title">{$title}</h3>
      <badge id="Badge" hidden={$badge == ""}>{$badge}</badge>
    </div>
    <p id="Body" part="card.body" hidden={$body == ""}>{$body}</p>
    </div>
  </button>
  <footer id="Actions">{@render actions?.()}</footer>
</article>

<style>
#Card { display: flex; flex-direction: column; width: var(--measure-sm); gap: var(--space-sm); }
.hit { padding: var(--space-md); width: stretch; }
#Face { display: flex; flex-direction: column; gap: var(--space-sm); width: stretch; }
#Image { width: stretch; aspect-ratio: 1.778; border-radius: var(--radius-control); }
#Head { display: flex; align-items: center; gap: var(--space-sm); }
#Title { width: stretch; }
#Body { margin: 0; color: var(--ink-muted); line-clamp: 3; }
#Actions { display: flex; justify-content: end; gap: var(--space-sm); }
</style>
