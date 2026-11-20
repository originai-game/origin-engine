<meta name="component" content="BlocksPage">

<!-- A game's own shooter HUD, laid out by the game from the HUD's blocks
     alone (HudPortrait, HudBar, HudSlot, HudEffect, HudHint and
     KillfeedRow), unlike the combat page's ready-made plates: the squad as
     a column of slim frames at the top left, the killfeed at the top right,
     the player's frame with its effects over it at the bottom left, the kit
     as one strip at the foot with the ultimate at its centre, and the input
     hints at the bottom right. The page owns every place and size (the
     blocks' size tokens); the blocks only draw. The art is sample art of
     the sheet only. -->
<div id="Page">
  <img id="Stage" src={$stage} alt="">
  <div id="Scrim"></div>

  <ul id="Squad">
    {#each $squad as m (m.id)}
    <li class="mate" data-state={m.state}>
      <HudPortrait portrait={m.portrait} dimmed={m.state == 'out'} />
      <div class="mate-bars">
        <span class="mate-name">{m.name}</span>
        {#if m.shieldMax > 0}<div class="mate-shield"><HudBar value={m.shield} max={m.shieldMax} kind="shield" ghost={0} segment={25} low={false} showValue={false} name="" /></div>{/if}
        <div class="mate-health"><HudBar value={m.health} max={m.healthMax} kind="health" ghost={m.ghost} segment={0} low={false} showValue={false} name="" /></div>
      </div>
    </li>
    {/each}
  </ul>

  <ul id="Feed">
    {#each $kills as r (r.id)}<li><KillfeedRow killer={r.killer} victim={r.victim} weapon={r.weapon} kind={r.kind} headshot={r.headshot} self={r.self} team={r.team} /></li>{/each}
  </ul>

  <div id="Player">
    <ul id="Effects">{#each $effects as e (e.id)}<li><HudEffect name={e.name} icon={e.icon} glyph={e.glyph} remaining={e.remaining} label={e.label} harmful={e.harmful} /></li>{/each}</ul>
    <div id="Frame">
      <div id="PlayerPortrait"><HudPortrait portrait={$vitals.portrait} dimmed={false} /></div>
      <div id="PlayerBars">
        <div id="Reading"><span class="figure">{$vitals.health}</span><span class="most">/ {$vitals.healthMax}</span></div>
        <div id="PlayerShield"><HudBar value={$vitals.shield} max={$vitals.shieldMax} kind="shield" ghost={0} segment={$vitals.segment} low={false} showValue={false} name="" /></div>
        <div id="PlayerHealth"><HudBar value={$vitals.health} max={$vitals.healthMax} kind="health" ghost={$vitals.ghost} segment={0} low={false} showValue={false} name="" /></div>
      </div>
    </div>
  </div>

  <ul id="Kit">
    {#each $abilities as s (s.id)}<li class="slot" data-ultimate={s.ultimate}><HudSlot name={s.name} icon={s.icon} glyph={s.glyph} key={s.key} kind={s.ultimate ? 'ultimate' : s.count > 0 ? 'item' : 'skill'} cooldown={s.ultimate ? 0 : s.cooldown} time={s.time} count={s.count} ready={s.ready} unaffordable={s.disabled} reason="" charges={0} chargesMax={0} charge={s.charge} /></li>{/each}
  </ul>

  <div id="Hints"><HudHint key="F" label="Revive" /><HudHint key="V" label="Ping" /><HudHint key="Tab" label="Map" /></div>
</div>

<style>
#Page { position: fixed; inset: 0; overflow: hidden; background: #05070c; color: var(--hud-ink);
  --hud-accent: rgb(255 181 71); --hud-slot-ring: rgb(255 181 71); --hud-effect-harm-ink: var(--hud-foe);
  --hud-life-fill: var(--hud-health-fill); --hud-life-glow: var(--hud-health-glow); }
#Stage { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
#Scrim { position: absolute; left: 0; top: 0; width: stretch; height: stretch; background: linear-gradient(180deg, rgb(3 5 10 / 0.5), transparent 20%, transparent 70%, rgb(3 5 10 / 0.7)); }
ul { margin: 0; padding: 0; list-style: none; }

/* The squad: slim frames, a small portrait beside its name over its bars. */
#Squad { position: absolute; top: var(--space-lg); left: var(--space-xl); display: flex; flex-direction: column; gap: var(--space-sm); inline-size: calc(190 * var(--font-step-px)); max-inline-size: 40%; --hud-portrait-size: calc(34 * var(--font-step-px)); }
.mate { display: flex; align-items: center; gap: var(--space-xs); }
.mate[data-state=down], .mate[data-state=out] { opacity: 0.6; }
.mate-bars { display: flex; flex-direction: column; gap: calc(3 * var(--font-step-px)); flex: 1 1 0; min-inline-size: 0; }
.mate-name { font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: ellipsis; overflow: hidden; text-shadow: var(--hud-shadow); }
.mate-shield { --hud-bar-size: calc(4 * var(--font-step-px)); }
.mate-health { --hud-bar-size: calc(6 * var(--font-step-px)); }

/* The killfeed: rows against the right edge, newest on top. */
#Feed { position: absolute; top: var(--space-lg); right: var(--space-xl); display: flex; flex-direction: column; align-items: flex-end; gap: calc(4 * var(--font-step-px)); max-inline-size: 46%; }
#Feed > li { display: flex; justify-content: flex-end; max-inline-size: 100%; }

/* The player: the effects over a frame of a large portrait beside a
   shield cut into cells over a fat health bar with its figure. */
#Player { position: absolute; left: var(--space-xl); bottom: var(--space-lg); display: flex; flex-direction: column; gap: var(--space-xs); inline-size: calc(320 * var(--font-step-px)); max-inline-size: 40%; }
#Effects { display: flex; gap: calc(6 * var(--font-step-px)); padding-inline-start: calc(80 * var(--font-step-px)); }
#Frame { display: flex; align-items: center; gap: var(--space-sm); }
#PlayerPortrait { filter: var(--hud-lift); --hud-portrait-size: calc(66 * var(--font-step-px)); }
#PlayerBars { display: flex; flex-direction: column; gap: calc(4 * var(--font-step-px)); flex: 1 1 0; min-inline-size: 0; }
/* The figure stands over the bars, as a bright health fill would swallow it. */
#Reading { display: flex; align-items: baseline; justify-content: flex-end; gap: calc(4 * var(--font-step-px)); text-shadow: var(--hud-shadow); }
.figure { font-size: var(--hud-size-md); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); }
.most { font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); color: var(--hud-ink-dim); }
#PlayerShield { --hud-bar-size: calc(10 * var(--font-step-px)); }
#PlayerHealth { --hud-bar-size: calc(20 * var(--font-step-px)); }

/* The kit: one strip at the foot, the ultimate large at its centre. */
#Kit { position: absolute; bottom: var(--space-lg); left: 50%; translate: -50% 0; display: flex; align-items: flex-end; gap: var(--space-md); }
.slot { --hud-slot-size: calc(50 * var(--font-step-px)); }
.slot[data-ultimate=true] { --hud-slot-size: calc(84 * var(--font-step-px)); }

#Hints { position: absolute; right: var(--space-xl); bottom: var(--space-lg); display: flex; gap: var(--space-md); }

/* On a phone the player's frame spans the foot over the kit, the squad
   and the feed keep their corners, and the hints give way to touch. */
@media (width < 600px) {
  #Squad { left: var(--space-md); top: calc(var(--space-xxl) * 2); inline-size: calc(130 * var(--font-step-px)); --hud-portrait-size: calc(26 * var(--font-step-px)); }
  #Feed { right: var(--space-md); top: var(--space-sm); max-inline-size: 70%; }
  #Feed > li:nth-child(n + 3) { display: none; }
  #Player { left: var(--space-md); right: var(--space-md); bottom: calc(var(--space-sm) + 96 * var(--font-step-px)); inline-size: auto; max-inline-size: none; }
  #Effects { padding-inline-start: 0; }
  #PlayerPortrait { --hud-portrait-size: calc(44 * var(--font-step-px)); }
  #PlayerHealth { --hud-bar-size: calc(14 * var(--font-step-px)); }
  #Kit { bottom: var(--space-sm); gap: var(--space-sm); }
  .slot { --hud-slot-size: calc(40 * var(--font-step-px)); }
  .slot[data-ultimate=true] { --hud-slot-size: calc(56 * var(--font-step-px)); }
  #Hints { display: none; }
}
</style>
