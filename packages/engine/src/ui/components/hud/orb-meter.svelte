<meta name="component" content="OrbMeter">
<meta name="strings" content="./orb-meter.en.ogt">
<meta name="parts" content="OrbMeter.orb OrbMeter.fill OrbMeter.value OrbMeter.label OrbMeter.well OrbMeter.figure OrbMeter.tag">
<meta name="prop:value" content="number" data-doc="The value.">
<meta name="prop:max" content="number" data-default="1" data-doc="Its maximum.">
<meta name="prop:kind" content="string" data-doc="Its liquid's colour: health (the style's --health) or mana (--mana).">
<meta name="prop:showValue" content="bool" data-doc="The value in the middle.">
<meta name="prop:label" content="string" data-doc="A caption under it; empty is none.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is the label, else Health.">
<meta name="fixture:default" content='{"value":340,"max":500,"kind":"health","showValue":true,"label":"Life","name":""}'>

<div id="OrbMeter" data-layout="stack" data-gap="tight" role="meter" aria-label={name || label || t("component.orb-meter.health")} aria-valuemin="0" aria-valuemax={max} aria-valuenow={value} class:mana={kind == 'mana'}>
  <div id="Orb" part="OrbMeter.orb OrbMeter.well" data-layout="overlay" style:--value={value / max}>
    <div id="Glass" data-anchor="fill"></div>
    <div id="Liquid" part="OrbMeter.fill" data-anchor="bottom"><div id="Surface"></div></div>
    <div id="Shine" data-anchor="fill"></div>
    {#if showValue}<span id="Value" class="value" part="OrbMeter.value OrbMeter.figure" data-anchor="center">{formatNumber(value)}</span>{/if}
  </div>
  {#if label}<span id="Label" class="label" part="OrbMeter.label OrbMeter.tag">{label}</span>{/if}
</div>

<style>
:host { text-shadow: var(--tag-halo); }
:host { align-items: center; --meter: var(--health); --fill-color: var(--meter); --layout-anchor-inset: 0; }
.mana { --meter: var(--mana); --fill-color: var(--meter); }
[part~="OrbMeter.orb"] { width: var(--size-xxl); height: var(--size-xxl); overflow: hidden; --well-radius: var(--radius-capsule); border: var(--stroke-thick) solid var(--line-strong); }
#Glass { background: radial-gradient(circle at 50% 40%, var(--panel-raised), var(--well)); }
[part="OrbMeter.fill"] { width: 100%; height: calc(var(--value) * 100%); background: linear-gradient(180deg, var(--sheen), transparent 30%, var(--shadow-45)), var(--fill-color); }
#Surface { height: var(--stroke-thick); background: var(--sheen); animation: sway var(--motion-duration-drift) var(--motion-easing-in-out) infinite alternate; }
#Shine { background: radial-gradient(ellipse 30% 16% at 40% 18%, var(--sheen), transparent), radial-gradient(circle at 50% 50%, transparent 62%, var(--shadow-45)); }
[part~="OrbMeter.value"] { --figure-size: var(--font-step-lg); --figure-ink: var(--scene-ink); --figure-shadow: var(--tag-halo); }
@keyframes sway { from { translate: calc(var(--space-xs) * -1) 0; } to { translate: var(--space-xs) 0; } }
</style>
