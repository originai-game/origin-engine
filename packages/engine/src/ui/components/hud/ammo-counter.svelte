<meta name="component" content="AmmoCounter">
<meta name="strings" content="./ammo-counter.en.ogt">
<meta name="prop:weapon" content="string" data-doc="The weapon's name.">
<meta name="prop:mode" content="string" data-default="auto" data-doc="Its fire mode: auto, burst or single (beside the name in words, and as three leaning pips: all lit for auto, two for burst, one for single).">
<meta name="prop:magazine" content="number" data-doc="Rounds in the magazine.">
<meta name="prop:capacity" content="number" data-doc="Rounds a full magazine holds: a leaning tick per round under the count, lit for each round left (a continuous strip past 60).">
<meta name="prop:reserve" content="number" data-doc="Rounds carried beyond the magazine.">
<meta name="prop:reloading" content="number" data-default="-1" data-doc="The reload's progress, 0 to 1; negative while not reloading.">
<meta name="prop:low" content="number" data-default="0.25" data-doc="The fraction of capacity at or under which the count turns low (0.25).">
<meta name="prop:secondary" content="string" data-doc="The weapon held in reserve, on a quiet line over the readout; empty hides the line.">
<meta name="prop:secondaryAmmo" content="number" data-default="-1" data-doc="The reserve weapon's rounds; negative hides the figure.">
<meta name="prop:secondaryKey" content="string" data-doc="The input that swaps to it as the game names it (2), on a key cap; empty hides it.">
<meta name="parts" content="Ammo.frame Ammo.alternate Ammo.swap Ammo.count Ammo.reserve Ammo.rounds Ammo.magazine Ammo.modes Ammo.weapon Ammo.mode Ammo.reload Ammo.prompt">
<meta name="fixture:default" content='{"weapon":"VK-7 Carbine","mode":"auto","magazine":24,"capacity":30,"reserve":120,"reloading":-1,"low":0.25,"secondary":"Hollow SMG","secondaryAmmo":32,"secondaryKey":"2"}'>
<meta name="fixture:low" content='{"weapon":"Brace-12","mode":"single","magazine":2,"capacity":6,"reserve":18,"reloading":-1,"low":0.34,"secondary":"","secondaryAmmo":-1,"secondaryKey":""}'>
<meta name="fixture:empty" content='{"weapon":"Talon AR","mode":"burst","magazine":0,"capacity":30,"reserve":0,"reloading":-1,"low":0.25,"secondary":"Wren Pistol","secondaryAmmo":15,"secondaryKey":"2"}'>
<meta name="fixture:reloading" content='{"weapon":"Wren Revolver","mode":"single","magazine":0,"capacity":6,"reserve":42,"reloading":0.55,"low":0.34,"secondary":"","secondaryAmmo":-1,"secondaryKey":""}'>
<meta name="fixture:belt" content='{"weapon":"Rampart LMG","mode":"auto","magazine":148,"capacity":200,"reserve":400,"reloading":-1,"low":0.25,"secondary":"","secondaryAmmo":-1,"secondaryKey":""}'>
<meta name="fixture:overflow" content='{"weapon":"Northern Relay Extended Marksman Rifle Mark Seven","mode":"burst","magazine":1024,"capacity":1024,"reserve":123456,"reloading":-1,"low":0.25,"secondary":"Hollow-Point Compact Submachine Gun","secondaryAmmo":12345,"secondaryKey":"Left Shoulder"}'>

<!--
  Ammunition at the screen's bottom corner: the reserve weapon on a quiet
  line, the magazine as the HUD's largest figure with the reserve beside
  it, a leaning tick per round (lit while loaded), and the fire mode as
  pips beside the weapon's name. Low and empty change the count's ink and
  say so; a reload shows its progress. The game owns every count; read
  only.
-->
<div id="AmmoCounter" part="Ammo.frame" class="hud bare" data-layout="stack" role="group" aria-label={t("shooter.ammo.name")} aria-valuetext={t("shooter.ammo.value", { magazine: magazine, capacity: capacity, reserve: reserve })} class:low={magazine > 0 && magazine <= capacity * low} class:empty={magazine <= 0}>
  {#if secondary != ''}<div id="Alternate" part="Ammo.alternate" data-layout="row" data-gap="tight">
    <span id="AlternateName" class="label">{secondary}</span>
    {#if secondaryAmmo >= 0}<span id="AlternateCount" class="small">{formatNumber(secondaryAmmo)}</span>{/if}
    {#if secondaryKey != ''}<span id="Swap" part="Ammo.swap" class="key">{secondaryKey}</span>{/if}
  </div>{/if}
  <div id="Rounds" data-layout="row">
    <span id="Count" part="Ammo.count" class:low={magazine > 0 && magazine <= capacity * low} class:empty={magazine <= 0}>{formatNumber(magazine)}</span>
    <span id="Reserve" part="Ammo.reserve">{formatNumber(reserve)}</span>
  </div>
  {#if capacity > 0}<div id="Ticks" part="Ammo.rounds" data-layout="overlay" class:belt={capacity > 60} style:--capacity={capacity > 60 ? 60 : capacity} style:--fill={magazine <= 0 ? 0 : magazine >= capacity ? 1 : magazine / capacity}><div id="Loaded" part="Ammo.magazine" data-anchor="top-start"></div></div>{/if}
  <div id="Weapon" data-layout="row" data-gap="tight">
    <div id="Modes" part="Ammo.modes" data-layout="row" data-gap="none" data-mode={mode}><i class="pip one"></i><i class="pip two"></i><i class="pip three"></i></div>
    <span id="Name" part="Ammo.weapon" class="label">{weapon}</span>
    {#if mode == 'auto'}<span id="Auto" class="label mode" part="Ammo.mode">{t("shooter.ammo.auto")}</span>{/if}
    {#if mode == 'burst'}<span id="Burst" class="label mode" part="Ammo.mode">{t("shooter.ammo.burst")}</span>{/if}
    {#if mode == 'single'}<span id="Single" class="label mode" part="Ammo.mode">{t("shooter.ammo.single")}</span>{/if}
  </div>
  {#if reloading >= 0}<div id="Reload" data-layout="row" data-gap="tight" style:--progress={reloading >= 1 ? 1 : reloading}>
    <span id="ReloadLabel" class="label">{t("shooter.ammo.reloading")}</span>
    <div id="ReloadTrack" data-layout="overlay"><div id="ReloadFill" part="Ammo.reload" data-anchor="top-start"></div></div>
  </div>{/if}
  {#if magazine <= 0 && reloading < 0}<span id="Prompt" part="Ammo.prompt" class="label">{t("shooter.ammo.empty")}</span>{/if}
</div>

<style>
:host { align-items: flex-end; row-gap: calc(5 * var(--font-step-px)); inline-size: calc(230 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; --layout-anchor-inset: 0; }
.label { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; }
/* The reserve weapon: a quiet line over the readout. */
#Alternate { align-items: center; max-inline-size: 100%; min-inline-size: 0; opacity: 0.8; }
#AlternateName { min-inline-size: 0; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.small { flex: none; font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.key { flex: 0 1 auto; min-inline-size: calc(20 * var(--font-step-px)); max-inline-size: 50%; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); text-align: center; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); -webkit-text-stroke-width: 0; text-shadow: none; white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
/* The magazine is the HUD's largest figure; the reserve sits at its foot, quieter. */
#Rounds { align-items: baseline; column-gap: var(--space-sm); max-inline-size: 100%; }
#Count { min-inline-size: 0; font-size: var(--hud-size-ammo); line-height: 0.9; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
#Count.low { color: var(--hud-low-ink); }
#Count.empty { color: var(--hud-empty-ink); }
#Reserve { flex: none; font-size: var(--hud-size-reserve); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink-dim); white-space: nowrap; }
/* A leaning tick per round: the strip is the spent rounds' ink, the lit
   rounds' share of it, from the start, is loaded; both are cut into ticks by one mask. A belt past 60 is one
   strip. */
#Ticks { inline-size: calc(var(--capacity) * 7 * var(--font-step-px)); max-inline-size: 100%; block-size: calc(14 * var(--font-step-px)); transform: skewX(var(--bar-skew)); background: var(--hud-pip-off); mask-image: repeating-linear-gradient(90deg, #000 0 calc(100% / var(--capacity) - 2 * var(--font-step-px)), transparent calc(100% / var(--capacity) - 2 * var(--font-step-px)) calc(100% / var(--capacity))); }
#Ticks.belt { block-size: calc(6 * var(--font-step-px)); mask-image: none; }
#Loaded { inline-size: calc(var(--fill) * 100%); block-size: 100%; background: var(--hud-ink); }
.low #Loaded { background: var(--hud-low-ink); }
#Weapon { align-items: center; max-inline-size: 100%; min-inline-size: 0; }
#Name { min-inline-size: 0; color: var(--hud-ink); text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.mode { flex: none; color: var(--hud-ink-dim); }
/* The fire mode's pips: all three lit for auto, two for burst, one for single. */
#Modes { flex: none; column-gap: calc(3 * var(--font-step-px)); }
.pip { inline-size: calc(10 * var(--font-step-px)); block-size: calc(4 * var(--font-step-px)); transform: skewX(var(--bar-skew)); background: var(--hud-pip-off); }
[data-mode=auto] .pip, [data-mode=burst] .one, [data-mode=burst] .two, [data-mode=single] .one { background: var(--hud-pip-on); box-shadow: var(--hud-pip-glow); }
#Reload { align-items: center; inline-size: 100%; }
#ReloadTrack { flex: 1; block-size: calc(4 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
#ReloadFill { inline-size: calc(var(--progress) * 100%); block-size: 100%; background: var(--hud-pip-on); }
#Prompt { color: var(--hud-empty-ink); animation: var(--pulse-motion); }
@media (width < 600px) {
  :host { inline-size: auto; row-gap: calc(2 * var(--font-step-px)); }
  #Modes { display: none; }
  #Weapon { flex-wrap: wrap; justify-content: flex-end; }
  #Count { font-size: var(--hud-size-ammo-compact); }
  #Reserve { font-size: var(--hud-size-sm); }
  #Ticks { block-size: calc(6 * var(--font-step-px)); inline-size: calc(var(--capacity) * 4 * var(--font-step-px)); }
  #Alternate .small { font-size: var(--hud-size-xs); }
}
/* Under a touch screen there is no key to swap with. */
@media (pointer: coarse) { #Swap { display: none; } }
@media (prefers-reduced-motion: reduce) { #Prompt { animation: none; } }
</style>
