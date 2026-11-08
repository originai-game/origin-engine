<meta name="component" content="objective-tracker">
<meta name="prop:title" content="string" data-doc="The quest or chapter.">
<meta name="prop:objectives" content="list" data-doc="A keyed collection of { id, label, progress (0–1), done }.">
<meta name="fixture:default" content="{&quot;title&quot;:&quot;The Lighthouse&quot;,&quot;objectives&quot;:[{&quot;id&quot;:&quot;a&quot;,&quot;label&quot;:&quot;Find the keeper&quot;,&quot;progress&quot;:1,&quot;done&quot;:true},{&quot;id&quot;:&quot;b&quot;,&quot;label&quot;:&quot;Light the lamps 2/5&quot;,&quot;progress&quot;:0.4,&quot;done&quot;:false}]}">

<section id="Tracker" role="status" aria-label={$title}>
  <h3 id="Title">{$title}</h3>
  <ul id="Objectives">
    {#each $objectives as o (o.id)}
      <li class="objective" class:done={o.done} style:--p={o.progress}>
        <p class="label">{o.label}</p>
        <div class="bar"><div class="fill"></div></div>
      </li>
    {/each}
  </ul>
</section>

<style>
#Tracker { display: flex; flex-direction: column; gap: 6px; width: 260px; }
#Title { margin: 0; }
#Objectives { list-style: none; padding: 0; margin: 0; gap: 6px; }
.label { margin: 0; }
.done .label { color: var(--ink-muted); }
.bar { height: 4px; width: stretch; background: var(--well); border-radius: 2px; }
.fill { height: 4px; width: calc(var(--p) * 100%); background: var(--accent); border-radius: 2px; }
</style>
