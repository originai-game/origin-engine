<meta name="component" content="Killfeed">
<meta name="strings" content="./shooter.en.ogt">
<meta name="prop:rows" content="list" data-doc="The eliminations to show, newest first, a keyed collection of { id, killer, victim, weapon, headshot, self, team }: self the player took part (the row is emphasised); team ally or enemy, the killer's side (the victim takes the other). The game trims old rows; a new row slides in.">
<meta name="parts" content="Killfeed.row Killfeed.name Killfeed.weapon Killfeed.headshot Killfeed.label">
<meta name="fixture:default" content='{"rows":[{"id":"k3","killer":"Kestrel","victim":"Vanta","weapon":"Halcyon","headshot":true,"self":true,"team":"ally"},{"id":"k2","killer":"Halden","victim":"Moraine","weapon":"Brace-12","headshot":false,"self":false,"team":"enemy"},{"id":"k1","killer":"Sable","victim":"Quill","weapon":"Talon AR","headshot":false,"self":false,"team":"ally"}]}'>
<meta name="fixture:empty" content='{"rows":[]}'>
<meta name="fixture:long" content='{"rows":[{"id":"l1","killer":"AVeryLongPlayerNameThatOverflows","victim":"AnotherExtremelyLongName_42","weapon":"Lancet DMR","headshot":true,"self":false,"team":"enemy"}]}'>

<div id="Killfeed" role="log" aria-label={t("shooter.killfeed.name")}>
  {#each rows as r (r.id)}
    <div class="row" part="Killfeed.row" class:self={r.self} aria-label={t("shooter.killfeed.row", { killer: r.killer, victim: r.victim, weapon: r.weapon })} in:fly={{ x: 24, duration: 200 }} out:fade={{ duration: 200 }} animate:flip={{ duration: 200 }}>
      <span class="name killer" part="Killfeed.name" class:ally={r.team == 'ally'} class:enemy={r.team != 'ally'}>{r.killer}</span>
      <span class="weapon" part="Killfeed.weapon Killfeed.label">{r.weapon}</span>
      {#if r.headshot}<og-icon class="headshot" part="Killfeed.headshot" icon="skull" aria-label={t("shooter.killfeed.headshot")}></og-icon>{/if}
      <span class="name victim" part="Killfeed.name" class:ally={r.team != 'ally'} class:enemy={r.team == 'ally'}>{r.victim}</span>
    </div>
  {/each}
</div>

<style>
:host { display: flex; flex-direction: column; align-items: flex-end; gap: var(--space-xs); pointer-events: none; max-width: 100%; text-shadow: var(--tag-halo); }
.row { display: flex; align-items: center; gap: var(--space-sm); padding: var(--space-xs) var(--space-sm); border-radius: var(--radius-control); background: var(--scene-plate); width: fit-content; max-width: 100%; border-inline-end: var(--stroke-regular) solid var(--line-strong); }
.row.self { border-inline-end-color: var(--primary); background: var(--hud); }
.name { font-size: var(--font-step-sm); line-height: var(--font-leading-normal); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: calc(var(--size-xxl) * 1.6); color: var(--scene-ink); text-shadow: 0 var(--stroke-hairline) var(--stroke-regular) var(--scene-shadow); }
/* Sides by place: the killer always leads the row; the pack inks each
   name by team (Killfeed::name.ally, .enemy). */
/* A phone shows the three newest, upright or on its side (where six rows
   would cover the upper corner of the play area); the rest stay in the log
   for a reader. */
@media (width < 600px) { .row:nth-child(n + 4) { display: none; } }
@media (height < 600px) and (orientation: landscape) { .row:nth-child(n + 4) { display: none; } }
.weapon { font-size: var(--font-step-sm); line-height: var(--font-leading-normal); white-space: nowrap; text-transform: uppercase; }
.headshot { width: var(--size-xs); height: var(--size-xs); color: var(--scene-ink); }
.self .killer { font-weight: 600; }
</style>
