<meta name="component" content="key-values">
<meta name="prop:rows" content="list" data-doc="The rows, a keyed collection of { id, key, value, delta } (keyValueRows; statCompare when delta is set: + better, - worse).">
<meta name="fixture:default" content="{&quot;rows&quot;:[{&quot;id&quot;:&quot;atk&quot;,&quot;key&quot;:&quot;Attack&quot;,&quot;value&quot;:&quot;42&quot;,&quot;delta&quot;:&quot;+6&quot;},{&quot;id&quot;:&quot;def&quot;,&quot;key&quot;:&quot;Defence&quot;,&quot;value&quot;:&quot;18&quot;,&quot;delta&quot;:&quot;-2&quot;},{&quot;id&quot;:&quot;wt&quot;,&quot;key&quot;:&quot;Weight&quot;,&quot;value&quot;:&quot;3.5 kg&quot;,&quot;delta&quot;:&quot;&quot;}]}">

<ul id="KeyValues">
  {#each $rows as r (r.id)}
    <li class="row">
      <p class="key">{r.key}</p>
      <p class="value">{r.value}</p>
      <p class="delta" class:up={r.delta != "" && r.delta.0 == "+"} hidden={r.delta == ""}>{r.delta}</p>
    </li>
  {/each}
</ul>

<style>
#KeyValues { list-style: none; padding: 0; margin: 0; width: stretch; }
.row { display: flex; align-items: center; gap: 8px; padding: 4px 0; border-bottom: 1px solid var(--line); }
.key { width: stretch; margin: 0; color: var(--ink-muted); }
.value { margin: 0; }
.delta { margin: 0; width: 48px; text-align: end; color: var(--danger); }
.delta.up { color: var(--success); }
</style>
