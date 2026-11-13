<meta name="component" content="data-list">
<meta name="parts" content="data-list.frame data-list.row data-list.label data-list.figure data-list.note">
<meta name="prop:rows" content="list" data-doc="The rows, a keyed collection of { id, title, detail, value }.">
<meta name="prop:loading" content="bool" data-doc="Rows are on their way: a busy line instead of the empty one.">
<meta name="prop:empty" content="string" data-doc="What an empty list says.">
<meta name="prop:busy" content="string" data-doc="What a loading list says.">
<meta name="action:choose" content="Open">
<meta name="fixture:default" content="{&quot;loading&quot;:false,&quot;empty&quot;:&quot;Nothing here yet&quot;,&quot;busy&quot;:&quot;Loading…&quot;,&quot;rows&quot;:[{&quot;id&quot;:&quot;a&quot;,&quot;title&quot;:&quot;Copper ore&quot;,&quot;detail&quot;:&quot;Mined in the north pit&quot;,&quot;value&quot;:&quot;×12&quot;},{&quot;id&quot;:&quot;b&quot;,&quot;title&quot;:&quot;Silk&quot;,&quot;detail&quot;:&quot;Bought in Harrow&quot;,&quot;value&quot;:&quot;×3&quot;}]}">

<section id="DataList" part="data-list.frame" class="bare" aria-busy={$loading}>
  <list id="Rows">
    {#each $rows as r (r.id)}
      <button id="Row" part="data-list.row" class="row" onclick={choose}>
        <div class="face">
          <div class="text"><p class="title">{r.title}</p><small class="detail" part="data-list.label">{r.detail}</small></div>
          <p class="value" part="data-list.figure">{r.value}</p>
        </div>
      </button>
    {:else}
      <p id="Empty" part="data-list.note" class="note">{$loading ? $busy : $empty}</p>
    {/each}
  </list>
</section>

<style>
#DataList { width: stretch; height: stretch; }
#Rows { width: stretch; height: stretch; gap: var(--space-xs); }
.row { width: stretch; padding: var(--space-sm) var(--space-md); }
.face { display: flex; align-items: center; gap: var(--space-md); width: stretch; }
.text { width: stretch; gap: var(--space-xxs); }
.title, .value { margin: 0; }
.note { color: var(--ink-muted); margin: var(--space-lg) 0; text-align: center; width: stretch; }
</style>
