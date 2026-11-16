<meta name="component" content="OrbMeter">
<meta name="strings" content="./orb-meter.en.ogt">
<meta name="parts" content="OrbMeter.orb OrbMeter.fill OrbMeter.value OrbMeter.label">
<meta name="prop:value" content="number" data-doc="The value.">
<meta name="prop:max" content="number" data-default="1" data-doc="Its maximum.">
<meta name="prop:kind" content="string" data-doc="Its liquid's colour: health (the style's --health) or mana (--mana).">
<meta name="prop:showValue" content="bool" data-doc="The value in the middle.">
<meta name="prop:label" content="string" data-doc="A caption under it; empty is none.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is the label, else Health.">
<meta name="fixture:default" content='{"value":340,"max":500,"kind":"health","showValue":true,"label":"Life","name":""}'>

<div id="OrbMeter" role="meter" aria-label={name || label || t("component.orb-meter.health")} aria-valuemin="0" aria-valuemax={max} aria-valuenow={value} class:mana={kind == 'mana'}>
  <div id="Orb" part="OrbMeter.orb" style:--value={value / max}>
    <div id="Glass"></div>
    <div id="Liquid" part="OrbMeter.fill"><div id="Surface"></div></div>
    <div id="Shine"></div>
    {#if showValue}<span id="Value" part="OrbMeter.value">{formatNumber(value)}</span>{/if}
  </div>
  {#if label}<span id="Label" part="OrbMeter.label">{label}</span>{/if}
</div>

<style>
:host { text-shadow: var(--tag-halo); }
:host { display: flex; flex-direction: column; align-items: center; gap: var(--space-xs); --meter: var(--health); }
.mana { --meter: var(--mana); }
[part="OrbMeter.orb"] { position: relative; display: grid; width: var(--size-xxl); height: var(--size-xxl); align-items: center; justify-items: center; overflow: hidden; border: var(--stroke-thick) solid var(--line-strong); border-radius: var(--radius-capsule); }
#Glass, #Shine { position: absolute; inset: 0; }
#Glass { background: radial-gradient(circle at 50% 40%, var(--panel-raised), var(--well)); }
[part="OrbMeter.fill"] { position: absolute; inset-inline: 0; bottom: 0; height: calc(var(--value) * 100%); background: linear-gradient(180deg, var(--sheen), transparent 30%, var(--shadow-45)), var(--meter); }
#Surface { height: var(--stroke-thick); background: var(--sheen); animation: sway var(--motion-duration-drift) var(--motion-easing-in-out) infinite alternate; }
#Shine { background: radial-gradient(ellipse 30% 16% at 40% 18%, var(--sheen), transparent), radial-gradient(circle at 50% 50%, transparent 62%, var(--shadow-45)); }
[part="OrbMeter.value"] { position: relative; font: var(--text-lg); font-family: var(--figure-font); color: var(--scene-ink); text-shadow: var(--tag-halo); }
[part="OrbMeter.label"] { font: var(--text-sm); color: var(--scene-ink); text-shadow: 0 var(--stroke-hairline) var(--stroke-regular) var(--scene-shadow); }
@keyframes sway { from { translate: calc(var(--space-xs) * -1) 0; } to { translate: var(--space-xs) 0; } }
</style>
