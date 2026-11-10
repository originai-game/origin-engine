<meta name="component" content="StatBar">
<meta name="strings" content="./stat-bar.en.ogt">
<meta name="prop:value" content="number" data-doc="The value.">
<meta name="prop:max" content="number" data-doc="Its maximum.">
<meta name="prop:kind" content="string" data-doc="What it measures (its colour): health, mana, stamina or xp; health turns warn and danger as it falls.">
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
<meta name="prop:low" content="number" data-doc="The fraction under which it is low (0.3).">
<meta name="prop:critical" content="number" data-doc="The fraction under which it is critical (0.15).">
<meta name="prop:bare" content="bool" data-doc="No HUD plate behind it.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is the label, else Health.">
<meta name="fixture:default" content='{"value":72,"max":100,"kind":"health","segment":25,"shield":60,"shieldMax":100,"heal":12,"recoverable":0,"label":"HP","icon":"heart","showValue":true,"valueInside":false,"valueText":"","low":0.3,"critical":0.15,"bare":false,"name":""}'>

<div id="StatBar" role="meter" aria-label={name || label || t("component.stat-bar.health")} aria-valuemin="0" aria-valuemax={max} aria-valuenow={value} aria-valuetext={valueText || t("component.stat-bar.value", { value: value, max: max })} class:mana={kind == 'mana'} class:stamina={kind == 'stamina'} class:xp={kind == 'xp'} class:low={kind == 'health' && value / max <= low && value / max > critical} class:critical={kind == 'health' && value / max <= critical} class:bare={bare}>
  <div id="Shield" hidden={!(shieldMax > 0)} style:--shield={shield / shieldMax}><div id="ShieldFill"></div></div>
  <div id="Main">
    <og-icon id="Icon" icon={icon} hidden={!icon}></og-icon>
    <span id="Label" hidden={!label}>{label}</span>
    <div id="Track" style:--value={value / max} style:--heal={heal / max} style:--recoverable={recoverable / max} style:--blocks={"repeating-linear-gradient(90deg, #00000000 0% " + (segment / max * 100 - 0.8) + "%, #000000cc " + (segment / max * 100 - 0.8) + "% " + (segment / max * 100) + "%)"}>
      <div id="Recoverable"></div>
      <div id="Ghost"></div>
      <div id="Heal" hidden={!(heal > 0)}></div>
      <div id="Fill"></div>
      <div id="Blocks" hidden={!(segment > 0)}></div>
      <div id="Gloss"></div>
      <span id="Inside" hidden={!(showValue && valueInside)}>{valueText || t("component.stat-bar.value", { value: value, max: max })}</span>
    </div>
    <span id="Value" hidden={!showValue || valueInside}>{valueText || t("component.stat-bar.value", { value: value, max: max })}</span>
  </div>
</div>

<style>
#StatBar { width: 240px; gap: 3px; --meter: var(--health); }
#StatBar.mana { --meter: var(--mana); }
#StatBar.stamina { --meter: var(--success); }
#StatBar.xp { --meter: var(--xp); }
#StatBar.low { --meter: var(--warn); }
#StatBar.critical { --meter: var(--danger); }
#Shield { width: stretch; height: 5px; border-radius: 2px; background: #00000066; overflow: hidden; }
#ShieldFill { width: calc(var(--shield) * 100%); height: stretch; background: var(--info); }
#Main { display: flex; width: stretch; gap: 8px; align-items: center; padding: 4px 8px 4px 6px; border-radius: 6px; background: var(--hud); }
.bare #Main { padding: 0; background: #00000000; }
#Icon { width: 20px; height: 20px; color: var(--meter); }
#Label { font: var(--text-sm); white-space: nowrap; text-shadow: 0 1px 2px #000000cc; }
#Track { width: stretch; height: 14px; border-radius: 4px; background: var(--control-strong); overflow: hidden; box-shadow: inset 0 1px 2px #00000088; }
#Recoverable, #Ghost, #Fill, #Heal, #Blocks, #Gloss { position: absolute; top: 0; height: stretch; }
#Recoverable { inset-inline-start: 0; width: calc(var(--recoverable) * 100%); background: var(--ink-muted); opacity: 0.55; }
#Ghost { inset-inline-start: 0; width: stretch; background: #fff2d9; opacity: 0.85; scale: var(--value) 1; transform-origin: 0% 50%; transition: scale 650ms ease-in-out 450ms; }
#Fill { inset-inline-start: 0; width: calc(var(--value) * 100%); background: linear-gradient(180deg, #ffffff38, #00000000 50%, #0000002e), var(--meter); }
#Heal { inset-inline-start: 0; width: calc((var(--value) + var(--heal)) * 100%); background: var(--success); opacity: 0.5; animation: pulse 900ms ease-in-out infinite alternate; }
#Blocks { inset-inline-start: 0; width: stretch; background: var(--blocks); }
/* Transforms and gradients are physical: right to left, the ghost shrinks towards
   the right and the blocks count from the right, as the fill does. */
#StatBar:dir(rtl) #Ghost { transform-origin: 100% 50%; }
#StatBar:dir(rtl) #Blocks { scale: -1 1; }
#Gloss { inset-inline-start: 0; width: stretch; height: 42%; background: linear-gradient(180deg, #ffffff38, #ffffff00); }
#Inside { position: absolute; inset-inline-end: 6px; top: 0; height: stretch; font: var(--text-caption); font-family: var(--font-mono); text-shadow: 0 1px 2px #000000; }
#Value { font: var(--text-sm); font-family: var(--font-mono); white-space: nowrap; text-shadow: 0 1px 2px #000000cc; }
.critical #Value { color: var(--danger); }
.critical #Fill { animation: pulse 700ms ease-in-out infinite alternate; }
@keyframes pulse { from { opacity: 1; } to { opacity: 0.7; } }
</style>
