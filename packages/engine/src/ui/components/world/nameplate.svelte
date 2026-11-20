<meta name="component" content="Nameplate">
<meta name="strings" content="./world.en.ogt">
<meta name="parts" content="Nameplate.frame Nameplate.level Nameplate.icon Nameplate.name Nameplate.subtitle Nameplate.state Nameplate.health Nameplate.ghost Nameplate.fill Nameplate.shield Nameplate.percent Nameplate.wing Nameplate.bracket Nameplate.cast Nameplate.spell Nameplate.time Nameplate.castbar">
<meta name="prop:entity" content="number" data-doc="The unit it sits over, by its bits (0: use at).">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="A world point it sits over without an entity (x y z).">
<meta name="prop:height" content="number" data-doc="World units above the point (its head).">
<meta name="prop:name" content="string" data-doc="The name, in the reaction's ink; a long one scrolls on one line.">
<meta name="prop:title" content="string" data-doc="A line under it in small capitals (a role, a guild); empty is none.">
<meta name="prop:level" content="string" data-doc="Its level on a chip as the game words it (32); empty is none, a boss's empty level draws a skull.">
<meta name="prop:reaction" content="string" data-doc="Its colour: neutral (empty), friendly, hostile or player.">
<meta name="prop:health" content="number" data-default="-1" data-doc="Health 0 to 1 for a skewed bar cut in quarters under the name; negative draws none (a friendly NPC that cannot be hurt). When it drops, a cream ghost of what was lost holds 400 ms and drains in 300 ms. At 0.25 or less the rim turns danger and breathes, and the percent shows.">
<meta name="prop:shield" content="number" data-doc="An absorb after the health, 0 to 1, in cyan.">
<meta name="prop:castName" content="string" data-doc="The spell it is casting; empty is nothing. A boss or a caster sits above the interaction prompt when plates overlap.">
<meta name="prop:castProgress" content="number" data-doc="How far the cast is, 0 to 1 (the game's progress, read every tick).">
<meta name="prop:castTime" content="string" data-doc="The cast's time left as the game words it (1.2); empty is none.">
<meta name="prop:castState" content="string" data-doc="The cast: casting (empty), locked (it can't be interrupted: steel grey with a shield mark) or interrupted (a red hatch and the word; the game shows it 600 ms, then clears castName).">
<meta name="prop:icon" content="string" data-doc="A class, quest or role icon before the name; empty is none.">
<meta name="prop:elite" content="bool" data-doc="An elite: a gold level chip, gold wings at the bar's ends and a gold rim.">
<meta name="prop:boss" content="bool" data-doc="A boss: as an elite, with its name in tracked capitals, a longer bar and the percent.">
<meta name="prop:targeted" content="bool" data-doc="The player's target: 1.14 times larger, its name and rim white, white brackets that close in 120 ms; drawn over the rest.">
<meta name="prop:dead" content="bool" data-doc="Dead: the name greys and the bars go.">
<meta name="prop:lootable" content="bool" data-doc="Dead with loot: Lootable in gold under the name.">
<meta name="prop:compact" content="bool" data-doc="Far or crowded: only the bar (no name, title or cast).">
<meta name="prop:maxDistance" content="number" data-default="80" data-doc="Hidden past this many world units (80).">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","height":2.1,"name":"Vorric the Hollow","title":"Ashbound Ritualist","level":"43","reaction":"hostile","health":0.91,"shield":0.18,"castName":"Shadow Volley","castProgress":0.64,"castTime":"1.2","castState":"","icon":"","elite":true,"boss":false,"targeted":false,"dead":false,"lootable":false,"compact":false,"maxDistance":80}'>
<meta name="fixture:friendly" content='{"name":"Mira Ashgrove","title":"Warden’s Scout","level":"","reaction":"friendly","health":-1,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false}'>
<meta name="fixture:neutral" content='{"name":"Old Tam","title":"Ferryman","level":"12","reaction":"","health":1,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false}'>
<meta name="fixture:hostile" content='{"name":"Ashbound Thrall","title":"","level":"41","reaction":"hostile","health":0.78,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false}'>
<meta name="fixture:player" content='{"name":"Sylvenne","title":"Ninefold Wardens","level":"42","reaction":"player","health":0.86,"shield":0.1,"castName":"","castProgress":0,"castTime":"","elite":false}'>
<meta name="fixture:boss" content='{"name":"Ashen Colossus","title":"Warden of the Ninefold Gate","level":"","reaction":"hostile","health":0.58,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false,"boss":true}'>
<meta name="fixture:targeted" content='{"name":"Ashbound Thrall","title":"","level":"41","reaction":"hostile","health":0.78,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false,"targeted":true}'>
<meta name="fixture:low" content='{"name":"Ashbound Thrall","title":"","level":"41","reaction":"hostile","health":0.22,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false}'>
<meta name="fixture:casting" content='{"castState":""}'>
<meta name="fixture:locked" content='{"name":"Ashen Colossus","title":"Warden of the Ninefold Gate","level":"","health":0.58,"shield":0,"castName":"Cataclysm","castProgress":0.62,"castTime":"2.4","castState":"locked","elite":false,"boss":true,"targeted":true}'>
<meta name="fixture:interrupted" content='{"castProgress":1,"castTime":"","castState":"interrupted"}'>
<meta name="fixture:dead" content='{"name":"Ashbound Thrall","title":"","level":"41","reaction":"hostile","health":0,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false,"dead":true,"lootable":true}'>
<meta name="fixture:compact" content='{"name":"Ashbound Thrall","title":"","level":"41","reaction":"hostile","health":0.62,"shield":0,"castName":"","castProgress":0,"castTime":"","elite":false,"compact":true}'>
<meta name="fixture:overflow" content='{"name":"Ysolde the Unbroken, Warden of the Ninefold Gate","title":"Guardian of the Eastern Marches and the Ninefold Gate","level":"120","reaction":"hostile","health":0.12,"shield":0.15,"castName":"Ritual of the Ninefold Gate Unending","castProgress":0.85,"castTime":"12.4","castState":"","elite":false,"boss":true,"targeted":true}'>

<!--
  A unit's plate over its head: the level chip and the name in its
  reaction's ink, its title in small capitals, its health as a skewed bar
  cut in quarters (a cream ghost of a hit, the shield after the fill) and
  the cast row (the spell and its time over a thin bar). Read only; the
  game owns every figure and state.
-->
<anchor id="Nameplate" entity={entity || null} at={at} offset={"0 " + height + " 0"} occlusion="fade" clamp="contain" clamp-margin="24" declutter="stack" priority={targeted ? 1100 : (boss || castName != '' ? 400 : 100)} max-distance={maxDistance}>
  <div id="Plate" part="Nameplate.frame" class="bare" aria-label={name} data-reaction={reaction == '' ? 'neutral' : reaction} data-elite={elite || boss} data-boss={boss} data-targeted={targeted} data-low={health >= 0 && health <= 0.25 && !dead} data-dead={dead} data-compact={compact}>
    {#if !compact}
      <div class="head">
        {#if level != ''}<div part="Nameplate.level" class="level" aria-label={t("world.nameplate.level", { level: level })}><span>{level}</span></div>{:else}{#if boss}<div part="Nameplate.level" class="level" aria-label={t("world.nameplate.boss")}><og-icon class="skull" icon="skull"></og-icon></div>{/if}{/if}
        {#if icon}<og-icon part="Nameplate.icon" class="icon" icon={icon}></og-icon>{/if}
        <span part="Nameplate.name" class="name">{name}</span>
      </div>
      {#if title != ''}<span part="Nameplate.subtitle" class="subtitle">{title}</span>{/if}
      {#if dead && lootable}<span part="Nameplate.state" class="state" role="status">{t("world.nameplate.lootable")}</span>{/if}
    {/if}
    {#if health >= 0 && !dead}
      <div class="vitals">
        {#if (elite || boss) && !compact && !targeted}<span part="Nameplate.wing" class="wing"></span>{/if}
        <div part="Nameplate.health" class="bar" role="meter" aria-label={t("world.nameplate.health")} aria-valuemin="0" aria-valuemax="1" aria-valuenow={health} aria-valuetext={formatPercent(health)} style:--hp={health >= 1 ? 1 : health} style:--shield={shield}>
          <div class="well">
            <div part="Nameplate.ghost" class="ghost"></div>
            <div part="Nameplate.fill" class="fill"></div>
            {#if shield > 0}<div part="Nameplate.shield" class="shield"></div>{/if}
            <div class="quarters"></div>
          </div>
          {#if health <= 0.25}<div class="alarm"></div>{/if}
        </div>
        {#if (elite || boss) && !compact && !targeted}<span part="Nameplate.wing" class="wing end"></span>{/if}
        {#if (boss || health <= 0.25) && !compact}<span part="Nameplate.percent" class="percent">{formatPercent(health)}</span>{/if}
        {#if targeted}<div class="brackets"><span part="Nameplate.bracket" class="bracket"></span><span part="Nameplate.bracket" class="bracket end"></span></div>{/if}
      </div>
    {/if}
    {#if castName != '' && !compact && !dead}
      <div part="Nameplate.cast" class="cast" data-state={castState == '' ? 'casting' : castState} role="status" aria-label={castState == 'locked' ? t("world.nameplate.uninterruptible", { spell: castName }) : (castState == 'interrupted' ? t("world.nameplate.interrupted") : t("world.nameplate.casting", { spell: castName }))}>
        <div class="spell-line">
          <og-icon class="mark" icon={castState == 'locked' ? 'shield' : (castState == 'interrupted' ? 'close' : 'bolt')}></og-icon>
          <span part="Nameplate.spell" class="spell">{castState == 'interrupted' ? t("world.nameplate.interrupted") : castName}</span>
          {#if castTime != '' && castState != 'interrupted'}<span part="Nameplate.time" class="time">{castTime}</span>{/if}
        </div>
        <div part="Nameplate.castbar" class="castbar" style:--cast={castState == 'interrupted' ? 1 : (castProgress >= 1 ? 1 : castProgress)}><div class="cast-fill"></div></div>
      </div>
    {/if}
  </div>
</anchor>

<style>
/* Over the scene: one stroke under the fill and one soft shadow. */
#Plate { --bw: var(--world-bar-width); --bh: var(--world-bar-height); display: flex; flex-direction: column; align-items: center; row-gap: calc(2 * var(--font-step-px)); inline-size: max-content; max-inline-size: var(--world-nameplate-room); color: var(--hud-ink); line-height: 1.12; }
.name, .subtitle, .state, .percent, .spell-line { text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
#Plate[data-boss=true] { --bw: var(--world-boss-bar-width); --bh: var(--world-boss-bar-height); }
#Plate[data-compact=true] { --bw: var(--world-compact-bar-width); --bh: var(--world-compact-bar-height); }
#Plate[data-targeted=true] { scale: var(--world-target-scale); max-inline-size: calc(var(--world-nameplate-room) / var(--world-target-scale)); }
.head { display: flex; align-items: center; justify-content: center; column-gap: calc(4 * var(--font-step-px)); max-inline-size: 100%; min-inline-size: 0; }
/* The level chip: a dark tag; an elite's or a boss's gold. */
.level { flex: none; display: flex; align-items: center; justify-content: center; min-inline-size: calc(19 * var(--font-step-px)); block-size: calc(16 * var(--font-step-px)); padding: 0 calc(4 * var(--font-step-px)); box-sizing: border-box; font-size: var(--world-size-level); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); text-shadow: none; -webkit-text-stroke-width: 0; border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
[data-elite=true] .level { color: var(--world-elite-chip-ink); background: var(--world-elite-chip-fill); box-shadow: none; }
[data-dead=true] .level { opacity: 0.5; }
.skull { inline-size: calc(11 * var(--font-step-px)); block-size: calc(11 * var(--font-step-px)); }
.icon { flex: none; inline-size: calc(14 * var(--font-step-px)); block-size: calc(14 * var(--font-step-px)); }
/* The name in its reaction's ink; a boss's in tracked capitals; the
   target's white; a dead unit's grey. A long one scrolls on one line. */
.name { flex: 0 1 auto; min-inline-size: 0; max-inline-size: calc(150 * var(--font-step-px)); font-size: var(--world-size-name); font-weight: var(--hud-label-weight); font-stretch: var(--world-name-stretch); color: var(--world-neutral-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
[data-reaction=hostile] .name { color: var(--world-hostile-ink); }
[data-reaction=friendly] .name { color: var(--world-friendly-ink); }
[data-reaction=player] .name { color: var(--world-player-ink); }
[data-boss=true] .name { max-inline-size: calc(200 * var(--font-step-px)); font-size: var(--world-size-boss); font-weight: var(--hud-figure-weight); font-stretch: var(--world-caps-stretch); letter-spacing: var(--world-caps-tracking); text-transform: var(--hud-label-transform); }
[data-targeted=true] .name { color: var(--hud-ink); }
[data-dead=true] .name { color: var(--world-dead-ink); }
.subtitle { min-inline-size: 0; max-inline-size: min(100%, calc(190 * var(--font-step-px))); font-size: var(--world-size-title); font-weight: var(--hud-label-weight); letter-spacing: var(--world-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
[data-dead=true] .subtitle { color: var(--world-dead-ink); }
.state { font-size: var(--world-size-title); font-weight: var(--hud-figure-weight); letter-spacing: var(--world-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-gold); white-space: nowrap; }
/* The health: a skewed well with its rim; inside it the ghost, the fill,
   the shield and the quarter cuts. The bar is art and keeps its size. */
.vitals { position: relative; display: flex; align-items: center; column-gap: calc(5 * var(--font-step-px)); }
.bar { position: relative; flex: none; inline-size: var(--bw); block-size: var(--bh); transform: skewX(var(--bar-skew)); box-shadow: var(--world-bar-rim); }
[data-elite=true] .bar { box-shadow: var(--world-elite-rim); }
[data-targeted=true] .bar { box-shadow: var(--world-target-rim); }
[data-targeted=true][data-elite=true] .bar { box-shadow: var(--world-target-elite-rim); }
.well { position: absolute; inset: 0; overflow: hidden; background: var(--bar-well); }
.ghost, .fill, .shield, .quarters { position: absolute; top: 0; bottom: 0; left: 0; }
.ghost { inline-size: calc(var(--hp) * 100%); background: var(--world-ghost); transition: var(--world-ghost-motion); }
.fill { inline-size: calc(var(--hp) * 100%); background: var(--world-neutral-fill); }
[data-reaction=hostile] .fill { background: var(--world-hostile-fill); }
[data-reaction=friendly] .fill { background: var(--world-friendly-fill); }
[data-reaction=player] .fill { background: var(--world-player-fill); }
.shield { left: calc(var(--hp) * 100%); inline-size: calc(var(--shield) * 100%); background: var(--hud-shield-fill); }
.quarters { right: 0; background: var(--world-quarters); }
/* Low: the rim turns danger and breathes. */
.alarm { position: absolute; inset: 0; box-shadow: var(--world-low-rim); animation: var(--world-low-motion); }
.wing { flex: none; inline-size: var(--world-wing-width); block-size: var(--world-wing-height); background: var(--hud-gold); filter: var(--world-mark-shadow); clip-path: polygon(100% 0, 0 50%, 100% 100%, 70% 50%); }
.wing.end { scale: -1 1; }
[data-boss=true] .wing { inline-size: var(--world-boss-wing-width); block-size: var(--world-boss-wing-height); }
.percent { flex: none; font-size: var(--hud-tag-size); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
[data-low=true] .percent { color: var(--world-hostile-ink); }
/* The target's brackets, outside the bar; they close as it is chosen. */
.brackets { position: absolute; top: 0; bottom: 0; left: calc(-1 * var(--world-bracket-reach)); right: calc(-1 * var(--world-bracket-reach)); display: flex; align-items: center; justify-content: space-between; animation: var(--world-bracket-motion); }
.bracket { flex: none; inline-size: var(--world-bracket-width); block-size: var(--world-bracket-height); background: var(--hud-ink); filter: var(--world-mark-shadow); clip-path: polygon(0 0, 100% 50%, 0 100%, 35% 50%); }
.bracket.end { scale: -1 1; }
/* The cast row: the spell and its time over a thin bar, at least the
   health bar's width; a long spell scrolls. */
.cast { display: flex; flex-direction: column; row-gap: calc(1 * var(--font-step-px)); min-inline-size: calc(var(--bw) + var(--space-xs) + var(--space-xxs)); max-inline-size: min(100%, calc(var(--bw) + 140 * var(--font-step-px))); margin-top: calc(1 * var(--font-step-px)); }
.spell-line { display: flex; align-items: center; column-gap: calc(6 * var(--font-step-px)); min-inline-size: 0; font-size: var(--world-size-caps); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--world-caps-tracking); text-transform: var(--hud-label-transform); color: var(--world-cast-ink); white-space: nowrap; }
.mark { flex: none; inline-size: var(--world-mark-size); block-size: var(--world-mark-size); margin-inline-end: calc(-3 * var(--font-step-px)); }
.spell { flex: 1 1 auto; min-inline-size: 0; max-inline-size: calc(170 * var(--font-step-px)); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.time { flex: none; letter-spacing: 0; font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); }
.castbar { position: relative; block-size: var(--world-cast-height); overflow: hidden; background: var(--bar-well); box-shadow: var(--world-cast-rim); }
.cast-fill { position: absolute; left: 0; top: 0; bottom: 0; inline-size: calc(var(--cast) * 100%); background: var(--world-cast-fill); box-shadow: var(--world-cast-glow); }
[data-state=locked] .spell-line { color: var(--world-locked-ink); }
[data-state=locked] .castbar { box-shadow: var(--world-locked-rim); }
[data-state=locked] .cast-fill { background: var(--world-locked-fill); box-shadow: none; }
[data-state=interrupted] .spell-line { color: var(--hud-ink); }
[data-state=interrupted] .castbar { box-shadow: var(--world-interrupted-rim); }
[data-state=interrupted] .cast-fill { background: var(--world-interrupted-fill); box-shadow: none; }
/* On a phone the plate is a little denser. */
@media (width < 600px) {
  #Plate { --bw: calc(var(--world-bar-width) * var(--world-density-compact)); }
  #Plate[data-boss=true] { --bw: calc(var(--world-boss-bar-width) * var(--world-density-compact)); }
  #Plate[data-compact=true] { --bw: calc(var(--world-compact-bar-width) * var(--world-density-compact)); }
}
/* Reduced motion: the low rim holds still, the ghost and brackets jump. */
@media (prefers-reduced-motion: reduce) {
  .alarm, .brackets { animation: none; }
  .ghost { transition: none; }
}
</style>
