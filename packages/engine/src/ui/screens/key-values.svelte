<meta name="component" content="key-values">
<meta name="parts" content="key-values.frame key-values.row key-values.label key-values.figure key-values.delta">
<meta name="prop:rows" content="list" data-doc="The rows, a keyed collection of { id, key, value, delta } (keyValueRows; statCompare when delta is set: + better, - worse).">
<meta name="fixture:default" content="{&quot;rows&quot;:[{&quot;id&quot;:&quot;atk&quot;,&quot;key&quot;:&quot;Attack&quot;,&quot;value&quot;:&quot;42&quot;,&quot;delta&quot;:&quot;+6&quot;},{&quot;id&quot;:&quot;def&quot;,&quot;key&quot;:&quot;Defence&quot;,&quot;value&quot;:&quot;18&quot;,&quot;delta&quot;:&quot;-2&quot;},{&quot;id&quot;:&quot;wt&quot;,&quot;key&quot;:&quot;Weight&quot;,&quot;value&quot;:&quot;3.5 kg&quot;,&quot;delta&quot;:&quot;&quot;}]}">

<ul id="KeyValues" part="key-values.frame" class="bare">
  {#each $rows as r (r.id)}
    <li class="row" part="key-values.row">
      <p class="key" part="key-values.label">{r.key}</p>
      <p class="value" part="key-values.figure">{r.value}</p>
      <p class="delta" part="key-values.delta" class:up={r.delta != "" && r.delta.0 == "+"} hidden={r.delta == ""}>{r.delta}</p>
    </li>
  {/each}
</ul>

<style>
#KeyValues { list-style: none; padding: 0; margin: 0; width: stretch; }
.row { display: flex; align-items: center; gap: var(--space-sm); padding: var(--space-xs) 0; border-bottom: var(--stroke-hairline) solid var(--line); }
.key { width: stretch; margin: 0; }
.value { margin: 0; }
.delta { margin: 0; width: var(--size-lg); text-align: end; color: var(--trend-down); }
.delta.up { color: var(--trend-up); }
</style>
