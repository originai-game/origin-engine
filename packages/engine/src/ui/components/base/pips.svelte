<meta name="component" content="Pips">
<meta name="strings" content="./pips.en.ogt">
<meta name="parts" content="Pips.frame Pips.units Pips.unit Pips.empty Pips.full Pips.state Pips.loading Pips.caption Pips.figure Pips.badge">
<meta name="prop:label" content="string" data-default="" data-doc="The resource's name; empty uses Resource.">
<meta name="prop:icon" content="string" data-default="heart" data-doc="The atlas icon for a unit: heart, bullet, bolt or a game's own icon.">
<meta name="prop:items" content="list" data-doc="Keyed units { id, value, label }: value is the filled fraction, 0 to 1; label names the unit for accessibility. The game supplies capacity, ordering and fractional units, with no fixed maximum.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Waiting for resource data; units are replaced by Loading.">
<meta name="prop:disabled" content="bool" data-default="false" data-doc="The resource is unavailable; its values remain readable.">
<meta name="prop:tone" content="string" data-default="" data-doc="The full units' colour: empty is the style's pip colour (a life's red); success, warn, danger, info or primary names another.">
<meta name="fixture:default" content='{"label":"Lives","icon":"heart","tone":"","items":[{"id":"life-a","value":1,"label":"First life"},{"id":"life-b","value":0.5,"label":"Second life"},{"id":"life-c","value":0,"label":"Third life"}],"loading":false,"disabled":false}'>
<meta name="fixture:full" content='{"label":"Action points","icon":"bolt","tone":"warn","items":[{"id":"ap-a","value":1},{"id":"ap-b","value":1},{"id":"ap-c","value":1}]}'>
<meta name="fixture:empty" content='{"label":"Lives","items":[]}'>
<meta name="fixture:depleted" content='{"label":"Lives","items":[{"id":"last","value":0}]}'>
<meta name="fixture:loading" content='{"label":"Lives","loading":true}'>
<meta name="fixture:disabled" content='{"label":"Action points","icon":"bolt","tone":"warn","disabled":true,"items":[{"id":"ap-a","value":1}]}'>

<!-- A counted resource on one line: its name, then one icon per unit. An
     empty unit is its socket; a full one is cut to its fraction and wears
     the tone. Unavailable, the units lose their colour and dim, full ones
     still brighter than their sockets. -->
<div id="Pips" part="Pips.frame" class="bare" data-layout="cluster" class:off={disabled} role="group" aria-label={label || t("base.pips.resource")} aria-busy={loading} aria-disabled={disabled}>
  <div part="Pips.caption" class="label">{label || t("base.pips.resource")}</div>
  {#if loading}<div part="Pips.loading" data-layout="row" data-gap="tight"><Spinner size={18} name={t("base.pips.loading")} /><div part="Pips.state Pips.caption" role="status">{t("base.pips.loading")}</div></div>
  {:else}
    {#if items.length}<div part="Pips.units" data-layout="cluster" data-gap="hair">
      {#each items as item (item.id)}
        <div class="unit" part="Pips.unit" role="meter" aria-label={item.label || label || t("base.pips.resource")} aria-valuemin="0" aria-valuemax="1" aria-valuenow={item.value < 0 ? 0 : item.value > 1 ? 1 : item.value} aria-valuetext={formatPercent(item.value < 0 ? 0 : item.value > 1 ? 1 : item.value)} style:--value={item.value}>
          <og-icon part="Pips.empty" class="empty" icon={icon}></og-icon>
          <og-icon part="Pips.full Pips.figure" class="full" data-tone={disabled ? null : tone || null} icon={icon}></og-icon>
        </div>
      {/each}
    </div>{:else}<div part="Pips.state Pips.caption">{t("base.pips.empty")}</div>{/if}
    {#if disabled}<div part="Pips.state Pips.badge">{t("base.pips.disabled")}</div>{/if}
  {/if}
</div>

<style>
:host { max-inline-size: 100%; }
.unit { position: relative; inline-size: var(--pip-size, var(--size-sm)); block-size: var(--pip-size, var(--size-sm)); }
.empty, .full { position: absolute; inset: 0; inline-size: 100%; block-size: 100%; }
.empty { color: var(--pip-empty-ink); }
.off .full { --figure-ink: var(--ink-label); }
.full { --figure-ink: var(--pip-ink); clip-path: inset(0 calc((1 - clamp(0, var(--value), 1)) * 100%) 0 0); transition: var(--pip-motion); }
</style>
