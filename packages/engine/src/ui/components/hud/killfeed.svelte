<meta name="component" content="Killfeed">
<meta name="strings" content="./killfeed.en.ogt">
<meta name="prop:rows" content="list" data-doc="The eliminations to show, newest first, a keyed collection of { id, killer, victim, weapon, kind, headshot, self, team }: kind the weapon's class (rifle, smg, pistol, shotgun, sniper, launcher or melee) drawn as its glyph, empty for the weapon's name in words; self the player took part (the row's edge is lit and the player's name takes the HUD's accent); team ally or enemy, the killer's side (the victim takes the other). The game trims old rows (a phone shows the two newest upright, three on its side); a new row slides in and an old one stays whole until it fades out over its last 300 ms.">
<meta name="parts" content="Killfeed.row Killfeed.name Killfeed.weapon Killfeed.glyph Killfeed.headshot Killfeed.edge">
<meta name="fixture:default" content='{"rows":[{"id":"k3","killer":"You","victim":"Vanta","weapon":"VK-7 Carbine","kind":"rifle","headshot":true,"self":true,"team":"ally"},{"id":"k2","killer":"Quill","victim":"Okonkwo-Reyes","weapon":"Hollow SMG","kind":"smg","headshot":false,"self":false,"team":"ally"},{"id":"k1","killer":"Vanta","victim":"Moraine","weapon":"Talon AR","kind":"rifle","headshot":false,"self":false,"team":"enemy"}]}'>
<meta name="fixture:words" content='{"rows":[{"id":"w2","killer":"Halden","victim":"Kestrel","weapon":"Brace-12","kind":"","headshot":true,"self":true,"team":"enemy"},{"id":"w1","killer":"Sable","victim":"Quill","weapon":"Pike .50","kind":"sniper","headshot":false,"self":false,"team":"ally"}]}'>
<meta name="fixture:empty" content='{"rows":[]}'>
<meta name="fixture:long" content='{"rows":[{"id":"l1","killer":"AVeryLongPlayerNameThatOverflows","victim":"AnotherExtremelyLongName_42","weapon":"Lancet DMR Extended Marksman Rifle","kind":"","headshot":true,"self":false,"team":"enemy"}]}'>

<!--
  Eliminations at the screen's top corner, newest first: each row the
  killer, the weapon's glyph (or its name), the victim and a headshot tag,
  names in their side's colour on a dark glass that fades toward the
  middle, an edge at the screen's side lit for the player's own rows.
  Read only.
-->
<div id="Killfeed" role="log" aria-label={t("shooter.killfeed.name")}>
  {#each rows as r (r.id)}
    <div class="row" part="Killfeed.row" data-layout="row" data-gap="tight" class:self={r.self} aria-label={t("shooter.killfeed.row", { killer: r.killer, victim: r.victim, weapon: r.weapon })} in:fly={{ x: 24, duration: 200 }} out:fade={{ duration: 300 }} animate:flip={{ duration: 200 }}>
      <span class="name killer" part="Killfeed.name" class:ally={r.team == 'ally'} class:enemy={r.team != 'ally'}>{r.killer}</span>
      {#if r.kind == 'rifle' || r.kind == 'smg' || r.kind == 'pistol' || r.kind == 'shotgun' || r.kind == 'sniper' || r.kind == 'launcher' || r.kind == 'melee'}
      <svg class="glyph" part="Killfeed.glyph" viewBox="0 0 46 16" aria-label={r.weapon}>
        {#if r.kind == 'rifle'}<path d="M0 5h30V3h6v2h10v3H34l-2 3h-6l-1-3h-9l-3 7H8l2-7H0z"></path>{/if}
        {#if r.kind == 'smg'}<path d="M2 6h38V4h4v6H22l-2 5h-6l1-5H2z"></path>{/if}
        {#if r.kind == 'pistol'}<path d="M12 3h24v5H25l-2 7h-6l2-7h-7z"></path>{/if}
        {#if r.kind == 'shotgun'}<path d="M0 5h44v3H20l-3 6h-6l3-6H0z"></path>{/if}
        {#if r.kind == 'sniper'}<path d="M0 7h46v2H30l-2 6h-5l1-6H0zM17 3h13v3H17z"></path>{/if}
        {#if r.kind == 'launcher'}<path d="M2 3h40v8H28l-2 4h-6l1-4H2z"></path>{/if}
        {#if r.kind == 'melee'}<path d="M4 8l30-5 9 4-9 4-30-1z"></path>{/if}
      </svg>
      {:else}<span class="weapon" part="Killfeed.weapon">{r.weapon}</span>{/if}
      <span class="name victim" part="Killfeed.name" class:ally={r.team != 'ally'} class:enemy={r.team == 'ally'}>{r.victim}</span>
      {#if r.headshot}<span class="headshot" part="Killfeed.headshot">{t("shooter.killfeed.hs")}</span>{/if}
      <div class="edge" part="Killfeed.edge"></div>
    </div>
  {/each}
</div>

<style>
:host { display: flex; flex-direction: column; align-items: flex-end; row-gap: calc(5 * var(--font-step-px)); pointer-events: none; max-inline-size: 100%; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.row { align-items: center; max-inline-size: 100%; min-inline-size: 0; padding: 0; padding-inline-start: var(--space-md); background: var(--hud-feed-fill); }
.row:dir(rtl) { background: var(--hud-feed-fill-start); }
.name { flex: 0 1 auto; min-inline-size: 0; padding-block: calc(3 * var(--font-step-px)); font-size: var(--hud-size-feed); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
/* Sides by colour and by place: the killer always leads the row. */
.name.ally { color: var(--hud-ally); }
.name.enemy { color: var(--hud-foe); }
.self .killer.ally, .self .victim.ally { color: var(--hud-accent); }
.glyph { flex: none; inline-size: calc(40 * var(--font-step-px)); block-size: calc(14 * var(--font-step-px)); fill: var(--hud-glyph); filter: var(--hud-glyph-shadow); }
.weapon { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.headshot { flex: none; padding: 0 calc(5 * var(--font-step-px)); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-foe); white-space: nowrap; -webkit-text-stroke-width: 0; text-shadow: none; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
/* The edge at the screen's side, lit for the player's own rows. */
.edge { flex: none; align-self: stretch; inline-size: var(--hud-feed-edge-width); margin-inline-start: var(--space-xs); background: var(--hud-feed-edge); }
.self .edge { background: var(--hud-accent); }
/* An upright phone shows the newest row across its width, and a phone on
   its side, with the width for more, the three newest (where more rows
   would cover the play area round the aim); the rest stay in the log for
   a reader. */
@media (width < 600px) {
  .row:nth-child(n + 2) { display: none; }
  .row { padding-inline-start: var(--space-sm); }
  .name { font-size: var(--hud-size-xs); }
  .glyph { inline-size: calc(26 * var(--font-step-px)); block-size: calc(10 * var(--font-step-px)); }
}
@media (height < 600px) and (orientation: landscape) { .row:nth-child(n + 4) { display: none; } }
</style>
