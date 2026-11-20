<meta name="component" content="KillfeedRow">
<meta name="strings" content="./killfeed.en.ogt">
<meta name="parts" content="KillfeedRow.row KillfeedRow.name KillfeedRow.weapon KillfeedRow.glyph KillfeedRow.headshot KillfeedRow.edge">
<meta name="prop:killer" content="string" data-doc="Who made the elimination, as the game names them.">
<meta name="prop:victim" content="string" data-doc="Who was eliminated.">
<meta name="prop:weapon" content="string" data-doc="The weapon as the game names it (VK-7 Carbine): in words when kind is empty, and always what a reader hears.">
<meta name="prop:kind" content="string" data-doc="The weapon's class, drawn as its silhouette: rifle, smg, pistol, shotgun, sniper, launcher or melee; empty writes the weapon's name.">
<meta name="prop:headshot" content="bool" data-doc="A headshot: a skull after the victim.">
<meta name="prop:self" content="bool" data-doc="The player took part: the plate is tinted and its edge lit in the HUD's accent, and the player's name takes the accent.">
<meta name="prop:team" content="string" data-default="ally" data-doc="ally or enemy, the killer's side; the victim takes the other.">
<meta name="fixture:default" content='{"killer":"Quill","victim":"Okonkwo-Reyes","weapon":"Hollow SMG","kind":"smg","headshot":false,"self":false,"team":"ally"}'>
<meta name="fixture:self" content='{"killer":"You","victim":"Vanta","weapon":"VK-7 Carbine","kind":"rifle","headshot":true,"self":true,"team":"ally"}'>
<meta name="fixture:enemy" content='{"killer":"Vanta","victim":"Moraine","weapon":"Pike .50","kind":"sniper","headshot":false,"self":false,"team":"enemy"}'>
<meta name="fixture:words" content='{"killer":"Halden","victim":"Kestrel","weapon":"Brace-12","kind":"","headshot":true,"self":false,"team":"enemy"}'>
<meta name="fixture:long" content='{"killer":"AVeryLongPlayerNameThatOverflows","victim":"AnotherExtremelyLongName_42","weapon":"Lancet DMR Extended Marksman Rifle","kind":"","headshot":true,"self":false,"team":"enemy"}'>

<!--
  One elimination of a killfeed, a block a feed is built from: the
  killer, the weapon's silhouette (or its name) and the victim, names in
  their side's colour, a skull for a headshot, on a smoked plate that
  fades toward the middle with an edge at the screen's side. The player's
  own row is tinted and lit in the HUD's accent. It is as wide as its
  content, up to its place. Read only.
-->
<div id="KillfeedRow" part="KillfeedRow.row" data-layout="row" data-gap="tight" role="group" class:self={self} aria-label={t("shooter.killfeed.row", { killer: killer, victim: victim, weapon: weapon })}>
  <span class="name killer" part="KillfeedRow.name" class:ally={team == 'ally'} class:enemy={team != 'ally'}>{killer}</span>
  {#if kind == 'rifle' || kind == 'smg' || kind == 'pistol' || kind == 'shotgun' || kind == 'sniper' || kind == 'launcher' || kind == 'melee'}
  <svg class="glyph" part="KillfeedRow.glyph" viewBox="0 0 64 20" aria-label={weapon}>
    {#if kind == 'rifle'}<path d="M2 6h10l2-2h26l1-2h5v2h16v3H46v1h-8l-2 2h-3l-2 8h-6l2-8h-5l-2 3h-4l-1-4h-3l-7 5H2z"></path>{/if}
    {#if kind == 'smg'}<path d="M6 6h30V4h8v2h14v3H36l-2 2h-4l-1 7h-5l1-7h-5l-1 4h-5l1-4-5-2-4 3H6z"></path>{/if}
    {#if kind == 'pistol'}<path d="M16 4h32v5H34l-1 2h-3l-2 7h-8l3-9h-7z"></path>{/if}
    {#if kind == 'shotgun'}<path d="M1 8l9-2h40V4h12v4H50v2H36V8H22l-5 6h-6l2-5-12 3z"></path>{/if}
    {#if kind == 'sniper'}<path d="M1 9l11-1h8V6h2V3h16v3h2v2h23v2H36l-2 2h-4l-3 6h-5l2-6h-8l-6 3H1z"></path>{/if}
    {#if kind == 'launcher'}<path d="M4 4h52l4-2h2v10h-2l-4-2H30l-2 7h-6l1-7H12l-1 6H6l1-6H4z"></path>{/if}
    {#if kind == 'melee'}<path d="M4 9l16-2 2-2h2v2h26l10 3-10 2H24v2h-2l-2-2-16-1z"></path>{/if}
  </svg>
  {:else}<span class="weapon" part="KillfeedRow.weapon">{weapon}</span>{/if}
  <span class="name victim" part="KillfeedRow.name" class:ally={team != 'ally'} class:enemy={team == 'ally'}>{victim}</span>
  {#if headshot}<og-icon class="headshot" part="KillfeedRow.headshot" icon="skull" aria-label={t("shooter.killfeed.headshot")}></og-icon>{/if}
  <div class="edge" part="KillfeedRow.edge"></div>
</div>

<style>
:host { inline-size: fit-content; max-inline-size: 100%; min-inline-size: 0; align-items: center; column-gap: calc(7 * var(--font-step-px)); padding: 0; padding-inline-start: var(--space-md); color: var(--hud-ink); background: var(--hud-feed-fill); box-shadow: var(--hud-feed-rim); border-radius: var(--hud-feed-cut) 0 0 var(--hud-feed-cut); corner-shape: var(--cut-shape); clip-path: border-box; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
#KillfeedRow:dir(rtl) { background: var(--hud-feed-fill-start); border-radius: 0 var(--hud-feed-cut) var(--hud-feed-cut) 0; }
/* The player's own row: tinted in the accent, so it is found at a glance. */
#KillfeedRow.self { background: var(--hud-feed-self-fill); }
#KillfeedRow.self:dir(rtl) { background: var(--hud-feed-self-fill-start); }
.name { flex: 0 1 auto; min-inline-size: 0; padding-block: calc(4 * var(--font-step-px)); font-size: var(--hud-size-feed); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
/* Sides by colour and by place: the killer always leads the row. */
.name.ally { color: var(--hud-ally); }
.name.enemy { color: var(--hud-foe); }
.self .name.ally { color: var(--hud-accent); }
.glyph { flex: none; inline-size: calc(50 * var(--font-step-px)); block-size: calc(16 * var(--font-step-px)); fill: var(--hud-glyph); filter: var(--hud-glyph-shadow); }
.weapon { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.headshot { flex: none; inline-size: calc(16 * var(--font-step-px)); block-size: calc(16 * var(--font-step-px)); color: var(--hud-foe); filter: var(--hud-glyph-shadow); }
/* The edge at the screen's side, lit for the player's own rows. */
.edge { flex: none; align-self: stretch; inline-size: var(--hud-feed-edge-width); margin-inline-start: var(--space-xs); background: var(--hud-feed-edge); }
.self .edge { background: var(--hud-accent); box-shadow: 0 0 calc(8 * var(--font-step-px)) var(--hud-accent); }
@media (width < 600px), (height < 500px) {
  :host { padding-inline-start: var(--space-sm); column-gap: calc(5 * var(--font-step-px)); }
  .name { font-size: var(--hud-size-xs); padding-block: calc(3 * var(--font-step-px)); }
  .glyph { inline-size: calc(34 * var(--font-step-px)); block-size: calc(11 * var(--font-step-px)); }
  .headshot { inline-size: calc(12 * var(--font-step-px)); block-size: calc(12 * var(--font-step-px)); }
}
</style>
