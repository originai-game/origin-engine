<meta name="component" content="StatBar">
<meta name="strings" content="./stat-bar.en.ogt">
<meta name="parts" content="StatBar.shield StatBar.frame StatBar.label StatBar.track StatBar.fill StatBar.inside StatBar.figure">
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
  <div id="Shield" part="StatBar.shield" hidden={!(shieldMax > 0)} style:--shield={shield / shieldMax}><div id="ShieldFill"></div></div>
  <div id="Main" part="StatBar.frame" class:hud={!bare} class:bare={bare}>
    <og-icon id="Icon" icon={icon} hidden={!icon}></og-icon>
    <span id="Label" part="StatBar.label" hidden={!label}>{label}</span>
    <div id="Track" part="StatBar.track" style:--value={value / max} style:--heal={heal / max} style:--recoverable={recoverable / max} style:--block={segment / max}>
      <div id="Recoverable"></div>
      <div id="Ghost"></div>
      <div id="Heal" hidden={!(heal > 0)}></div>
      <div id="Fill" part="StatBar.fill"></div>
      <div id="Blocks" hidden={!(segment > 0)}></div>
      <div id="Gloss"></div>
      <span id="Inside" part="StatBar.inside" hidden={!(showValue && valueInside)}>{valueText || t("component.stat-bar.value", { value: value, max: max })}</span>
    </div>
    <span id="Value" part="StatBar.figure" hidden={!showValue || valueInside}>{valueText || t("component.stat-bar.value", { value: value, max: max })}</span>
  </div>
</div>

<style>
#StatBar { width: var(--size-6xl); gap: var(--space-xxs); --meter: var(--health); }
#StatBar.mana { --meter: var(--mana); }
#StatBar.stamina { --meter: var(--success); }
#StatBar.xp { --meter: var(--xp); }
#StatBar.low { --meter: var(--warn); }
#StatBar.critical { --meter: var(--danger); }
[part="StatBar.shield"] { width: stretch; height: var(--space-xs); border-radius: var(--meter-radius); background: var(--shadow-45); overflow: hidden; }
#ShieldFill { width: calc(var(--shield) * 100%); height: stretch; background: var(--info); }
[part="StatBar.frame"] { display: flex; width: stretch; gap: var(--space-sm); align-items: center; }
#Icon { width: var(--size-xs); height: var(--size-xs); color: var(--meter); }
[part="StatBar.label"] { font: var(--text-sm); font-family: var(--label-font); white-space: nowrap; text-shadow: 0 var(--stroke-hairline) var(--stroke-regular) var(--scene-shadow); }
[part="StatBar.track"] { width: stretch; height: var(--space-lg); border-radius: var(--meter-radius); background: var(--control-strong); overflow: hidden; box-shadow: var(--meter-shadow); }
#Recoverable, #Ghost, #Fill, #Heal, #Blocks, #Gloss { position: absolute; top: 0; height: stretch; }
#Recoverable { inset-inline-start: 0; width: calc(var(--recoverable) * 100%); background: var(--ink-muted); opacity: var(--opacity-muted); }
#Ghost { inset-inline-start: 0; width: stretch; background: var(--scene-ink); opacity: var(--opacity-soft); scale: var(--value) 1; transform-origin: 0% 50%; transition: scale var(--motion-duration-bounce) var(--motion-easing-in-out) var(--motion-duration-slow); }
#Fill { inset-inline-start: 0; width: calc(var(--value) * 100%); background: linear-gradient(180deg, var(--sheen), transparent 50%, var(--shadow-30)), var(--meter); }
#Heal { inset-inline-start: 0; width: calc((var(--value) + var(--heal)) * 100%); background: var(--success); opacity: var(--opacity-dim); animation-name: pulse; animation-duration: calc(var(--motion-duration-slow) * 2); animation-timing-function: var(--motion-easing-in-out); animation-iteration-count: infinite; animation-direction: alternate; }
#Blocks { inset-inline-start: 0; width: stretch; background: repeating-linear-gradient(90deg, transparent 0% calc(var(--block) * 100% - 0.8%), var(--scene-shadow) calc(var(--block) * 100% - 0.8%) calc(var(--block) * 100%)); }
/* Transforms and gradients are physical: right to left, the ghost shrinks towards
   the right and the blocks count from the right, as the fill does. */
#StatBar:dir(rtl) #Ghost { transform-origin: 100% 50%; }
#StatBar:dir(rtl) #Blocks { scale: -1 1; }
#Gloss { inset-inline-start: 0; width: stretch; height: 42%; background: linear-gradient(180deg, var(--sheen), transparent); }
[part="StatBar.inside"] { position: absolute; inset-inline-end: var(--space-xs); top: 0; height: stretch; font: var(--text-caption); font-family: var(--figure-font); text-shadow: 0 var(--stroke-hairline) var(--stroke-regular) var(--scene-shadow); }
[part="StatBar.figure"] { font: var(--text-sm); font-family: var(--figure-font); white-space: nowrap; text-shadow: 0 var(--stroke-hairline) var(--stroke-regular) var(--scene-shadow); }
.critical [part="StatBar.figure"] { color: var(--danger); }
.critical #Fill { animation-name: pulse; animation-duration: var(--motion-duration-bounce); animation-timing-function: var(--motion-easing-in-out); animation-iteration-count: infinite; animation-direction: alternate; }
@keyframes pulse { from { opacity: 1; } to { opacity: 0.7; } }
</style>
