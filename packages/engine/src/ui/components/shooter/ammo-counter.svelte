<meta name="component" content="AmmoCounter">
<meta name="strings" content="./shooter.en.ogt">
<meta name="prop:weapon" content="string" data-doc="The weapon's name.">
<meta name="prop:mode" content="string" data-doc="Its fire mode: auto, burst or single.">
<meta name="prop:magazine" content="number" data-doc="Rounds in the magazine.">
<meta name="prop:capacity" content="number" data-doc="Rounds a full magazine holds; the strip under the count shows magazine over it.">
<meta name="prop:reserve" content="number" data-doc="Rounds carried beyond the magazine.">
<meta name="prop:reloading" content="number" data-doc="The reload's progress, 0 to 1; negative while not reloading.">
<meta name="prop:low" content="number" data-doc="The fraction of capacity at or under which the count turns low (0.25).">
<meta name="parts" content="Ammo.frame Ammo.count Ammo.reserve Ammo.mode Ammo.reload">
<meta name="fixture:default" content='{"weapon":"Halcyon Carbine","mode":"auto","magazine":24,"capacity":28,"reserve":112,"reloading":-1,"low":0.25}'>
<meta name="fixture:low" content='{"weapon":"Brace-12","mode":"single","magazine":2,"capacity":6,"reserve":18,"reloading":-1,"low":0.34}'>
<meta name="fixture:empty" content='{"weapon":"Talon AR","mode":"burst","magazine":0,"capacity":30,"reserve":0,"reloading":-1,"low":0.25}'>
<meta name="fixture:reloading" content='{"weapon":"Wren Revolver","mode":"single","magazine":0,"capacity":6,"reserve":42,"reloading":0.55,"low":0.34}'>

<div id="AmmoCounter" part="Ammo.frame" role="group" aria-label={t("shooter.ammo.name")} aria-valuetext={t("shooter.ammo.value", { magazine: magazine, capacity: capacity, reserve: reserve })} class:low={magazine > 0 && magazine <= capacity * low} class:empty={magazine <= 0}>
  <div id="Rounds">
    <span id="Count" part="Ammo.count" class:low={magazine > 0 && magazine <= capacity * low} class:empty={magazine <= 0}>{formatNumber(magazine)}</span>
    <span id="Reserve" part="Ammo.reserve">{formatNumber(reserve)}</span>
  </div>
  <div id="Magazine" style:--fill={capacity > 0 ? magazine / capacity : 0}><div id="MagazineFill"></div></div>
  <div id="Weapon">
    <span id="Name">{weapon}</span>
    <span id="Auto" class="mode" part="Ammo.mode" hidden={mode != 'auto'}>{t("shooter.ammo.auto")}</span>
    <span id="Burst" class="mode" part="Ammo.mode" hidden={mode != 'burst'}>{t("shooter.ammo.burst")}</span>
    <span id="Single" class="mode" part="Ammo.mode" hidden={mode != 'single'}>{t("shooter.ammo.single")}</span>
  </div>
  <div id="Reload" hidden={reloading < 0} style:--progress={reloading}>
    <div id="ReloadTrack"><div id="ReloadFill" part="Ammo.reload"></div></div>
    <span id="ReloadLabel">{t("shooter.ammo.reloading")}</span>
  </div>
  <span id="Prompt" hidden={!(magazine <= 0 && reloading < 0)}>{t("shooter.ammo.empty")}</span>
</div>

<style>
#AmmoCounter { align-items: flex-end; gap: var(--space-xs); padding: var(--space-sm) var(--space-md); border-radius: var(--radius-sm); background: var(--scene-plate); color: var(--scene-ink); min-width: calc(var(--size-xxl) * 1.75); }
#Rounds { display: flex; align-items: baseline; gap: var(--space-sm); }
/* The magazine is the largest number on the HUD; the reserve sits beside it,
   a step down and quieter, as Apex and Forza set a value beside its unit. */
#Count { font: var(--text-display-md); font-family: var(--font-mono); color: var(--scene-ink); text-shadow: 0 max(1px, 0.04em) max(1px, 0.08em) #000000; }
#Count.low { color: var(--warn); }
#Count.empty { color: var(--danger); }
#Reserve { font: var(--text-lg); font-family: var(--font-mono); color: var(--scene-ink); text-shadow: 0 max(1px, 0.04em) max(1px, 0.08em) #000000; }
#Magazine { width: stretch; height: var(--stroke-thick); background: var(--sheen); }
#MagazineFill { width: calc(var(--fill) * 100%); height: stretch; background: var(--scene-ink); transition: width var(--motion-ui-duration) var(--motion-ui-easing); }
.low #MagazineFill { background: var(--warn); }
#Weapon { display: flex; align-items: baseline; gap: var(--space-sm); }
#Name { font: var(--text-meta); color: var(--scene-ink); white-space: nowrap; text-overflow: ellipsis; overflow: hidden; max-width: calc(var(--size-xxl) * 2); }
.mode { font: var(--text-caption); text-transform: uppercase; }
#Reload { width: stretch; gap: var(--space-xxs); align-items: flex-end; }
#ReloadTrack { width: stretch; height: var(--stroke-thick); background: var(--sheen); }
#ReloadFill { width: calc(var(--progress) * 100%); height: stretch; background: var(--scene-ink); }
#ReloadLabel { font: var(--text-caption); color: var(--scene-ink); text-transform: uppercase; }
#Prompt { font: var(--text-caption); color: var(--danger); text-transform: uppercase; }
</style>
