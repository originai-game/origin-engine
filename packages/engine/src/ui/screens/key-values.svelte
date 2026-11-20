<meta name="component" content="key-values">
<meta name="prop:rows" content="list" data-doc="The rows, a keyed collection of { id, key, value, delta } (keyValueRows; statCompare when delta is set: + better, - worse).">
<meta name="fixture:default" content="{&quot;rows&quot;:[{&quot;id&quot;:&quot;atk&quot;,&quot;key&quot;:&quot;Attack&quot;,&quot;value&quot;:&quot;42&quot;,&quot;delta&quot;:&quot;+6&quot;},{&quot;id&quot;:&quot;def&quot;,&quot;key&quot;:&quot;Defence&quot;,&quot;value&quot;:&quot;18&quot;,&quot;delta&quot;:&quot;-2&quot;},{&quot;id&quot;:&quot;wt&quot;,&quot;key&quot;:&quot;Weight&quot;,&quot;value&quot;:&quot;3.5 kg&quot;,&quot;delta&quot;:&quot;&quot;}]}">

<ul id="KeyValues" part="frame" variant="bare">
  {#each rows as r (r.id)}
    <li class="row" part="row">
      <p class="key" part="label">{r.key}</p>
      <p class="value" part="figure">{r.value}</p>
      <p class="delta" part="delta" class:up={r.delta[0] == "+"} class:down={r.delta[0] == "-" || r.delta[0] == "−"} hidden={!r.delta}>{r.delta}</p>
    </li>
  {/each}
</ul>

<style>
#KeyValues { list-style: none; padding: 0; margin: 0; width: stretch; }
/* Name on the left in dim ink, the value in front ink and tabular
   figures, hairline rules between, as the settings rows read. */
.row { display: flex; align-items: center; gap: var(--space-sm); min-block-size: calc(36 * var(--font-step-px)); padding: var(--space-xs) 0; border-bottom: var(--stroke-hairline) solid var(--system-tabs-rule); }
.key { flex: 1; min-width: 0; margin: 0; color: var(--front-ink-dim); }
.value { margin: 0; font-weight: var(--font-weight-semibold); font-variant-numeric: tabular-nums; color: var(--front-ink); }
/* A change in the better green or the worse red by its sign; unsigned, plain. */
.delta { margin: 0; min-width: var(--size-lg); text-align: end; font-variant-numeric: tabular-nums; color: var(--front-ink-dim); }
.delta.up { color: var(--trend-up); }
.delta.down { color: var(--trend-down); }
</style>
