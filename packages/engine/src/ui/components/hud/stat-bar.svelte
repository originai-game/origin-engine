<meta name="component" content="StatBar">
<meta name="strings" content="./stat-bar.en.ogt">
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

<div id="StatBar" layout="stack" gap="hair" role="meter" aria-label={name || label || t("component.stat-bar.health")} aria-valuemin="0" aria-valuemax={max} aria-valuenow={value} aria-valuetext={valueText || t("component.stat-bar.value", { value: value, max: max })} class:mana={kind == 'mana'} class:stamina={kind == 'stamina'} class:xp={kind == 'xp'} class:low={kind == 'health' && value / max <= low && value / max > critical} class:critical={kind == 'health' && value / max <= critical} class:bare={bare}>
  <div id="Main" part="frame" layout="row" variant={bare ? 'bare' : 'hud'}>
    {#if icon}<og-icon id="Icon" part="icon" icon={icon}></og-icon>{/if}
    {#if label}<div part="tag"><span id="Label" class="label" part="label">{label}</span></div>{/if}
    <div id="Track" part="track" layout="overlay" style:--shield={shieldMax > 0 ? shield / shieldMax : 0} style:--value={value / max} style:--heal={heal / max} style:--recoverable={recoverable / max} style:--block={segment / max}>
      <div id="Recoverable" part="fill" anchor="top-start"></div>
      <div id="Ghost" part="fill" anchor="top-start"></div>
      {#if heal > 0}<div id="Heal" part="fill" anchor="top-start"></div>{/if}
      <div id="Fill" part="fill" anchor="top-start"></div>
      {#if shieldMax > 0 && shield > 0}<div class="shield" id="Shield" part="fill" anchor="top-start"></div>{/if}
      {#if segment > 0}<div id="Blocks" part="notch" anchor="fill"></div>{/if}
      {#if showValue && valueInside}<div class="inside" id="Inside" part="tag" anchor="top-end" layout="row"><span class="inside figure" part="figure">{valueText || t("component.stat-bar.value", { value: value, max: max })}</span></div>{/if}
    </div>
    {#if showValue && !valueInside}<div part="tag"><span id="Value" class="value figure" part="figure">{valueText || t("component.stat-bar.value", { value: value, max: max })}</span></div>{/if}
  </div>
</div>

<style>
#StatBar { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
#StatBar { inline-size: 100%; --meter: var(--health); --fill-color: var(--meter); --layout-anchor-inset: 0; }
.mana { --meter: var(--mana); --fill-color: var(--meter); }
.stamina { --meter: var(--success); --fill-color: var(--meter); }
.xp { --meter: var(--xp); --fill-color: var(--meter); }
.low { --meter: var(--warn); --fill-color: var(--meter); }
.critical { --meter: var(--bar-danger); --fill-color: var(--meter); }
#Main { align-items: center; }
#Icon { width: var(--size-xs); height: var(--size-xs); }
.label { white-space: nowrap; }
/* The bar leans forward in a dark well with one lit rim; it grows with the
   player's text scale, so a value set inside it keeps its room
   (--font-step-px is one CSS px at 100 % text). */
#Track { min-inline-size: 0; block-size: calc(18 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); border-radius: 0; background: var(--bar-well); box-shadow: var(--bar-rim); }
.critical #Track { box-shadow: var(--bar-critical-rim); animation: var(--pulse-motion); }
#Recoverable, #Ghost, #Fill, #Heal { block-size: 100%; --fill-gloss: var(--bar-gloss); }
#Recoverable { inline-size: calc(var(--recoverable) * 100%); --fill-color: var(--ink-muted); }
/* A loss leaves a warm ghost that holds, then drains after the fill. */
#Ghost { inline-size: calc(var(--value) * 100%); background: var(--bar-ghost); transition: var(--bar-ghost-motion); }
#Fill { inline-size: calc(var(--value) * 100%); }
/* Healing on its way: striped past the fill. */
#Heal { inline-size: calc((var(--value) + var(--heal)) * 100%); background: var(--bar-pending); }
/* The shield lies along the top of the bar it guards, as long as its share
   of its maximum; a broken shield draws nothing. */
#Shield { inline-size: calc(var(--shield) * 100%); block-size: 34%; --fill-color: var(--bar-shield); --fill-gloss: none; }
#Blocks { background: var(--bar-notch); mask-image: repeating-linear-gradient(90deg, transparent 0% calc(var(--block) * 100% - 0.8%), #000 calc(var(--block) * 100% - 0.8%) calc(var(--block) * 100%)); }
#StatBar:dir(rtl) #Blocks { scale: -1 1; }
/* The value inside stands upright in the leaning bar. */
#Inside { align-items: center; block-size: 100%; padding-inline: var(--space-sm); line-height: 1; transform: skewX(calc(-1 * var(--bar-skew))); }
.figure { --figure-size: var(--font-step-sm); --figure-weight: var(--bar-figure-weight); font-stretch: var(--bar-figure-stretch); white-space: nowrap; }
.critical .figure { --figure-ink: var(--tone-danger); }
@media (prefers-reduced-motion: reduce) { .critical #Track { animation: none; } }
</style>
