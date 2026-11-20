<meta name="component" content="Killfeed">
<meta name="strings" content="./killfeed.en.ogt">
<meta name="prop:rows" content="list" data-doc="The eliminations to show, newest first, a keyed collection of { id, killer, victim, weapon, kind, headshot, self, team }: kind the weapon's class (rifle, smg, pistol, shotgun, sniper, launcher or melee) drawn as its glyph, empty for the weapon's name in words; self the player took part (the row's edge is lit and the player's name takes the HUD's accent); team ally or enemy, the killer's side (the victim takes the other). The game trims old rows (a phone shows the two newest upright, three on its side); a new row slides in and an old one stays whole until it fades out over its last 300 ms.">
<meta name="parts" content="Killfeed.row">
<meta name="fixture:default" content='{"rows":[{"id":"k3","killer":"You","victim":"Vanta","weapon":"VK-7 Carbine","kind":"rifle","headshot":true,"self":true,"team":"ally"},{"id":"k2","killer":"Quill","victim":"Okonkwo-Reyes","weapon":"Hollow SMG","kind":"smg","headshot":false,"self":false,"team":"ally"},{"id":"k1","killer":"Vanta","victim":"Moraine","weapon":"Talon AR","kind":"rifle","headshot":false,"self":false,"team":"enemy"}]}'>
<meta name="fixture:words" content='{"rows":[{"id":"w2","killer":"Halden","victim":"Kestrel","weapon":"Brace-12","kind":"","headshot":true,"self":true,"team":"enemy"},{"id":"w1","killer":"Sable","victim":"Quill","weapon":"Pike .50","kind":"sniper","headshot":false,"self":false,"team":"ally"}]}'>
<meta name="fixture:empty" content='{"rows":[]}'>
<meta name="fixture:long" content='{"rows":[{"id":"l1","killer":"AVeryLongPlayerNameThatOverflows","victim":"AnotherExtremelyLongName_42","weapon":"Lancet DMR Extended Marksman Rifle","kind":"","headshot":true,"self":false,"team":"enemy"}]}'>

<!--
  Eliminations at the screen's top corner, newest first, built from
  KillfeedRow: each row the killer, the weapon's silhouette (or its name),
  the victim and a skull for a headshot, names in their side's colour on a
  smoked plate that fades toward the middle, the player's own rows tinted
  and lit in the accent. Read only.
-->
<div id="Killfeed" role="log" aria-label={t("shooter.killfeed.name")}>
  {#each rows as r (r.id)}
    <div class="row" part="Killfeed.row" in:fly={{ x: 24, duration: 200 }} out:fade={{ duration: 300 }} animate:flip={{ duration: 200 }}><KillfeedRow killer={r.killer} victim={r.victim} weapon={r.weapon} kind={r.kind} headshot={r.headshot} self={r.self} team={r.team} /></div>
  {/each}
</div>

<style>
:host { display: flex; flex-direction: column; align-items: flex-end; row-gap: calc(4 * var(--font-step-px)); pointer-events: none; max-inline-size: 100%; }
.row { display: flex; justify-content: flex-end; max-inline-size: 100%; min-inline-size: 0; }
/* An upright phone shows the newest row across its width, and a phone on
   its side, with the width for more, the three newest (where more rows
   would cover the play area round the aim); the rest stay in the log for
   a reader. */
@media (width < 600px) { .row:nth-child(n + 2) { display: none; } }
@media (height < 600px) and (orientation: landscape) { .row:nth-child(n + 4) { display: none; } }
</style>
