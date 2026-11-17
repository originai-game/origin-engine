<meta name="component" content="StatBar">
<meta name="strings" content="./stat-bar.en.ogt">
<meta name="parts" content="StatBar.shield StatBar.frame StatBar.label StatBar.track StatBar.fill StatBar.inside StatBar.figure StatBar.well StatBar.icon StatBar.tag StatBar.notch">
<meta name="prop:value" content="number" data-doc="The value.">
<meta name="prop:max" content="number" data-default="1" data-doc="Its maximum.">
<meta name="prop:kind" content="string" data-default="health" data-doc="What it measures (its colour): health, mana, stamina or xp; health turns warn and danger as it falls.">
<meta name="prop:segment" content="number" data-doc="Value per block: a notch at each block's end (0: one bar).">
<meta name="prop:shield" content="number" data-doc="Shield or armour on top of the value, a strip over the bar.">
<meta name="prop:shieldMax" content="number" data-doc="The most shield; 0 hides the strip.">
<meta name="prop:heal" content="number" data-doc="Healing on its way, a pale preview past the fill.">
<meta name="prop:recoverable" content="number" data-doc="Damage that can still come back, drawn grey between the value and this.">
<meta name="prop:label" content="string" data-doc="A short name at its start (HP); empty is none.">
<meta name="prop:icon" content="string" data-doc="An icon of the atlas before it; empty is none.">
<meta name="prop:showValue" content="bool" data-doc="The value as text at its end.">
<meta name="prop:valueInside" content="bool" data-doc="The value inside the track at its end, so the bar keeps its length.">
<meta name="prop:valueText" content="string" data-doc="The value as text instead of value / max.">
<meta name="prop:low" content="number" data-default="0.3" data-doc="The fraction under which it is low (0.3).">
<meta name="prop:critical" content="number" data-default="0.15" data-doc="The fraction under which it is critical (0.15).">
<meta name="prop:bare" content="bool" data-doc="No HUD plate behind it.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is the label, else Health.">
<meta name="fixture:default" content='{"value":72,"max":100,"kind":"health","segment":25,"shield":60,"shieldMax":100,"heal":12,"recoverable":0,"label":"HP","icon":"heart","showValue":true,"valueInside":false,"valueText":"","low":0.3,"critical":0.15,"bare":false,"name":""}'>

<div id="StatBar" data-layout="stack" data-gap="hair" role="meter" aria-label={name || label || t("component.stat-bar.health")} aria-valuemin="0" aria-valuemax={max} aria-valuenow={value} aria-valuetext={valueText || t("component.stat-bar.value", { value: value, max: max })} class:mana={kind == 'mana'} class:stamina={kind == 'stamina'} class:xp={kind == 'xp'} class:low={kind == 'health' && value / max <= low && value / max > critical} class:critical={kind == 'health' && value / max <= critical} class:bare={bare}>
  {#if shieldMax > 0}<div id="Shield" part="StatBar.shield StatBar.track" data-layout="overlay" style:--shield={shield / shieldMax}><div id="ShieldFill" part="StatBar.fill" data-anchor="top-start" data-tone="info"></div></div>{/if}
  <div id="Main" part="StatBar.frame" data-layout="row" class:hud={!bare} class:bare={bare}>
    {#if icon}<og-icon id="Icon" part="StatBar.icon" icon={icon}></og-icon>{/if}
    {#if label}<div part="StatBar.tag"><span id="Label" class="label" part="StatBar.label">{label}</span></div>{/if}
    <div id="Track" part="StatBar.well StatBar.track" data-layout="overlay" data-fill="true" style:--value={value / max} style:--heal={heal / max} style:--recoverable={recoverable / max} style:--block={segment / max}>
      <div id="Recoverable" part="StatBar.fill" data-anchor="top-start"></div>
      <div id="Ghost" part="StatBar.fill" data-anchor="top-start" data-ghost="true"></div>
      {#if heal > 0}<div id="Heal" part="StatBar.fill" data-anchor="top-start" data-motion="pulse"></div>{/if}
      <div id="Fill" part="StatBar.fill" data-anchor="top-start" data-motion={kind == 'health' && value / max <= critical ? 'pulse' : null}></div>
      {#if segment > 0}<div id="Blocks" part="StatBar.notch" data-anchor="fill"></div>{/if}
      {#if showValue && valueInside}<div id="Inside" part="StatBar.inside StatBar.tag" data-anchor="top-end"><span class="inside" part="StatBar.figure">{valueText || t("component.stat-bar.value", { value: value, max: max })}</span></div>{/if}
    </div>
    {#if showValue && !valueInside}<div part="StatBar.tag"><span id="Value" class="value" part="StatBar.figure">{valueText || t("component.stat-bar.value", { value: value, max: max })}</span></div>{/if}
  </div>
</div>

<style>
:host { text-shadow: var(--tag-halo); }
:host { inline-size: 100%; --meter: var(--health); --fill-color: var(--meter); --layout-anchor-inset: 0; }
.mana { --meter: var(--mana); --fill-color: var(--meter); }
.stamina { --meter: var(--success); --fill-color: var(--meter); }
.xp { --meter: var(--xp); --fill-color: var(--meter); }
.low { --meter: var(--warn); --fill-color: var(--meter); }
.critical { --meter: var(--danger); --fill-color: var(--meter); }
/* The shield is a tab of the plate, on the straight run of its top edge. */
#Shield { block-size: var(--space-xs); margin-inline: var(--plate-radius); overflow: hidden; --track-fill: var(--plate-fill); }
#ShieldFill { inline-size: calc(var(--shield) * 100%); block-size: 100%; }
#Icon { width: var(--size-xs); height: var(--size-xs); }
[part="StatBar.label"] { white-space: nowrap; }
#Track { min-inline-size: 0; block-size: var(--space-lg); overflow: hidden; }
#Recoverable, #Ghost, #Fill, #Heal { block-size: 100%; }
#Recoverable { inline-size: calc(var(--recoverable) * 100%); --fill-color: var(--ink-muted); }
#Ghost { inline-size: calc(var(--value) * 100%); }
#Fill { inline-size: calc(var(--value) * 100%); }
#Heal { inline-size: calc((var(--value) + var(--heal)) * 100%); --fill-color: var(--tone-heal); }
#Blocks { mask-image: repeating-linear-gradient(90deg, transparent 0% calc(var(--block) * 100% - 0.8%), #000 calc(var(--block) * 100% - 0.8%) calc(var(--block) * 100%)); }
:host(:dir(rtl)) #Blocks { scale: -1 1; }
#Inside { padding-inline: var(--space-xs); }
[part="StatBar.figure"] { --figure-size: var(--font-step-sm); white-space: nowrap; }
.critical [part="StatBar.figure"] { --figure-ink: var(--tone-danger); }
</style>
