<meta name="component" content="OrbMeter">
<meta name="strings" content="./orb-meter.en.ogt">
<meta name="prop:value" content="number" data-doc="The value.">
<meta name="prop:max" content="number" data-doc="Its maximum.">
<meta name="prop:kind" content="string" data-doc="Its liquid's colour: health (red) or mana (blue).">
<meta name="prop:showValue" content="bool" data-doc="The value in the middle.">
<meta name="prop:label" content="string" data-doc="A caption under it; empty is none.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is the label, else Health.">
<meta name="fixture:default" content='{"value":340,"max":500,"kind":"health","showValue":true,"label":"Life","name":""}'>

<div id="OrbMeter" role="meter" aria-label={name || label || t("component.orb-meter.health")} aria-valuemin="0" aria-valuemax={max} aria-valuenow={value} class:mana={kind == 'mana'}>
  <stack id="Orb" style:--value={value / max}>
    <div id="Glass"></div>
    <div id="Liquid"><div id="Surface"></div></div>
    <div id="Shine"></div>
    <span id="Value" hidden={!showValue}>{formatNumber(value)}</span>
  </stack>
  <span id="Label" hidden={!label}>{label}</span>
</div>

<style>
#OrbMeter { align-items: center; gap: 4px; --meter: var(--health); }
#OrbMeter.mana { --meter: var(--mana); }
#Orb { width: 112px; height: 112px; align-items: center; justify-content: center; clip-path: circle(50%); border: 3px solid var(--line-strong); border-radius: 56px; }
#Glass { width: stretch; height: stretch; background: radial-gradient(circle at 50% 40%, #1a1f2a, #05070b); }
#Liquid { position: absolute; inset-inline-start: 0; bottom: 0; width: stretch; height: calc(var(--value) * 100%); background: linear-gradient(180deg, #ffffff30, #00000000 30%, #00000066), var(--meter); }
#Surface { width: stretch; height: 4px; background: #ffffff55; animation: sway 2400ms ease-in-out infinite alternate; }
#Shine { width: stretch; height: stretch; background: radial-gradient(ellipse at 35% 22%, #ffffff66, #ffffff00 40%); }
#Value { font: var(--text-lg); font-family: var(--font-mono); text-shadow: 0 1px 3px #000000; }
#Label { font: var(--text-sm); text-shadow: 0 1px 2px #000000cc; }
@keyframes sway { from { translate: -6px 0; } to { translate: 6px 0; } }
</style>
