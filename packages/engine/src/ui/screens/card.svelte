<meta name="component" content="card">
<meta name="prop:title" content="string" data-doc="Its heading.">
<meta name="prop:body" content="string" data-doc="A line or two under the heading; empty: none.">
<meta name="prop:image" content="string" data-doc="A picture the game registered (UiDesc.images) across its top; empty: none.">
<meta name="prop:badge" content="string" data-doc="A badge at its top end (New, -20%); empty: none.">
<meta name="action:open" content="Open">
<meta name="fixture:default" content="{&quot;title&quot;:&quot;Ember Blade&quot;,&quot;body&quot;:&quot;Burns on hit. Two-handed.&quot;,&quot;image&quot;:&quot;&quot;,&quot;badge&quot;:&quot;New&quot;}">

<article id="Card">
  <button id="Hit" class="hit" onclick={open} aria-label={$title}>
    <div id="Face">
    <img id="Image" src={$image} alt="" hidden={$image == ""}>
    <div id="Head">
      <h3 id="Title">{$title}</h3>
      <badge id="Badge" hidden={$badge == ""}>{$badge}</badge>
    </div>
    <p id="Body" hidden={$body == ""}>{$body}</p>
    </div>
  </button>
  <footer id="Actions">{@render actions?.()}</footer>
</article>

<style>
#Card { display: flex; flex-direction: column; width: 240px; gap: 8px; }
.hit { padding: 12px; width: stretch; }
#Face { display: flex; flex-direction: column; gap: 8px; width: stretch; }
#Image { width: stretch; aspect-ratio: 1.778; border-radius: var(--radius-control); }
#Head { display: flex; align-items: center; gap: 8px; }
#Title { margin: 0; width: stretch; }
#Body { margin: 0; color: var(--ink-muted); line-clamp: 3; }
#Actions { display: flex; justify-content: end; gap: 8px; }
</style>
