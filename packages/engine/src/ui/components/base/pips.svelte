<meta name="component" content="Pips">
<meta name="strings" content="./pips.en.ogt">
<meta name="parts" content="Pips.frame Pips.label Pips.units Pips.unit Pips.empty Pips.fill Pips.state">
<meta name="prop:label" content="string" data-default="" data-doc="The resource's name; empty uses Resource.">
<meta name="prop:icon" content="string" data-default="heart" data-doc="The atlas icon for a unit: heart, bullet, bolt or a game's own icon.">
<meta name="prop:items" content="list" data-doc="Keyed units { id, value, label }: value is the filled fraction, 0 to 1; label names the unit for accessibility. The game supplies capacity, ordering and fractional units, with no fixed maximum.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Waiting for resource data; units are replaced by Loading.">
<meta name="prop:disabled" content="bool" data-default="false" data-doc="The resource is unavailable; its values remain readable.">
<meta name="fixture:default" content='{"label":"Lives","icon":"heart","items":[{"id":"life-a","value":1,"label":"First life"},{"id":"life-b","value":0.5,"label":"Second life"},{"id":"life-c","value":0,"label":"Third life"}],"loading":false,"disabled":false}'>
<meta name="fixture:full" content='{"label":"Action points","icon":"bolt","items":[{"id":"ap-a","value":1},{"id":"ap-b","value":1},{"id":"ap-c","value":1}]}'>
<meta name="fixture:empty" content='{"label":"Lives","items":[]}'>
<meta name="fixture:depleted" content='{"label":"Lives","items":[{"id":"last","value":0}]}'>
<meta name="fixture:loading" content='{"label":"Lives","loading":true}'>
<meta name="fixture:disabled" content='{"label":"Action points","disabled":true,"items":[{"id":"ap-a","value":1}]}'>

<div id="Pips" part="Pips.frame" class="bare" class:unavailable={disabled} role="group" aria-label={label || t("base.pips.resource")} aria-busy={loading} aria-disabled={disabled}>
  <div part="Pips.label">{label || t("base.pips.resource")}</div>
  {#if loading}<div part="Pips.state" role="status">{t("base.pips.loading")}</div>
  {:else}
    <div part="Pips.units">
      {#each items as item (item.id)}
        <div class="unit" part="Pips.unit" role="meter" aria-label={item.label || label || t("base.pips.resource")} aria-valuemin="0" aria-valuemax="1" aria-valuenow={item.value < 0 ? 0 : item.value > 1 ? 1 : item.value} aria-valuetext={formatPercent(item.value < 0 ? 0 : item.value > 1 ? 1 : item.value)} style:--value={item.value}>
          <og-icon part="Pips.empty" icon={icon}></og-icon>
          <og-icon part="Pips.fill" icon={icon}></og-icon>
        </div>
      {/each}
    </div>
    {#if !items.length}<div part="Pips.state">{t("base.pips.empty")}</div>{/if}
    {#if disabled}<div part="Pips.state">{t("base.pips.disabled")}</div>{/if}
  {/if}
</div>

<style>
:host { display: flex; flex-direction: column; gap: var(--space-xs); max-inline-size: 100%; }
[part="Pips.label"] { font: var(--text-caption); }
[part="Pips.units"] { display: flex; flex-wrap: wrap; gap: var(--space-xs); }
.unit { position: relative; inline-size: var(--pip-size, var(--size-sm)); block-size: var(--pip-size, var(--size-sm)); }
[part="Pips.empty"], [part="Pips.fill"] { position: absolute; inset: 0; inline-size: 100%; block-size: 100%; }
[part="Pips.empty"] { color: var(--ink-faint); }
[part="Pips.fill"] { color: var(--pip-ink, var(--health)); clip-path: inset(0 calc((1 - clamp(0, var(--value), 1)) * 100%) 0 0); }
.unavailable .unit { opacity: var(--state-disabled-opacity); }
[part="Pips.state"] { font: var(--text-caption); color: var(--ink-muted); }
</style>
