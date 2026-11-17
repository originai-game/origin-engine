<meta name="component" content="Combo">
<meta name="strings" content="./combo.en.ogt">
<meta name="parts" content="Combo.frame Combo.heading Combo.figure Combo.multiplier Combo.timer Combo.state Combo.caption">
<meta name="prop:count" content="number" data-default="0" data-doc="The game's combo count, shown at zero too.">
<meta name="prop:multiplier" content="number" data-default="1" data-doc="The game's multiplier; it is not derived from count.">
<meta name="prop:remaining" content="number" data-default="-1" data-doc="Remaining window, 0 to 1; negative hides the timer. The game advances it.">
<meta name="prop:state" content="string" data-default="idle" data-doc="idle, active or broken; zero remaining does not determine gameplay results.">
<meta name="prop:label" content="string" data-default="" data-doc="A custom heading; empty uses Combo.">
<meta name="fixture:default" content='{"count":17,"multiplier":2.5,"remaining":0.65,"state":"active","label":""}'>
<meta name="fixture:idle" content='{"count":0,"multiplier":1,"remaining":-1,"state":"idle"}'>
<meta name="fixture:broken" content='{"count":17,"multiplier":1,"remaining":0,"state":"broken"}'>
<meta name="fixture:overflow" content='{"count":1234567,"multiplier":12.5,"remaining":0.05,"state":"active","label":"Uninterrupted expedition chain"}'>

<div id="Combo" part="Combo.frame" class="hud" data-layout="stack" data-gap="tight" role="status" data-state={state}>
  <div part="Combo.heading">{label || t("hud.combo.name")}</div>
  <div class="figures" data-layout="cluster"><div class="count" part="Combo.figure">{formatNumber(count < 0 ? 0 : count)}</div><div part="Combo.multiplier Combo.figure">{t("hud.combo.multiplier", { value: multiplier })}</div></div>
  <div part="Combo.state Combo.caption" data-tone={state == 'broken' ? 'danger' : state == 'active' ? 'success' : null}>{state == 'broken' ? t("hud.combo.broken") : state == 'active' ? t("hud.combo.active") : t("hud.combo.idle")}</div>
  {#if remaining >= 0}<progress id="Timer" part="Combo.timer" value={remaining < 0 ? 0 : remaining > 1 ? 1 : remaining} aria-label={t("hud.combo.window")}></progress>{/if}
</div>

<style>
:host { max-inline-size: 100%; }
[part="Combo.heading"] { overflow-wrap: anywhere; }
.figures { align-items: baseline; }
.count { --figure-size: var(--font-step-2xl); }
[part~="Combo.multiplier"] { --figure-size: var(--font-step-lg); }
[part~="Combo.figure"] { overflow-wrap: anywhere; }
[part="Combo.timer"] { inline-size: 100%; }
</style>
