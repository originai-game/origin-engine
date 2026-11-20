<meta name="component" content="ShooterAbilities">
<meta name="strings" content="./shooter.en.ogt">
<meta name="parts" content="ShooterAbilities.frame ShooterAbilities.ability">
<meta name="prop:slots" content="list" data-doc="The abilities and items on the bar in the game's order, keyed rows of { id, name, icon, glyph, key, cooldown, time, count, ready, disabled, ultimate, charge }: name for a reader; icon a texture (0: the glyph, an icon name, in its place); key the input as the game names it (Q), on a key cap under the slot; cooldown the share of its cooldown still to run, 0 to 1 (a dark sweep over the art), with time its figure as the game words it (6); count the charges or items held, on a tag (×2; 0 hides it); ready lit (a lit edge and glow); disabled greyed. The ultimate (ultimate true) stands larger in a hexagon inside a ring that fills with its charge, 0 to 1, its share in figures until it is ready.">
<meta name="fixture:default" content='{"slots":[{"id":"dash","name":"Phase dash","icon":0,"glyph":"bolt","key":"Q","cooldown":0.38,"time":"6","count":0,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"frag","name":"Frag grenade","icon":0,"glyph":"bomb","key":"G","cooldown":0,"time":"","count":2,"ready":true,"disabled":false,"ultimate":false,"charge":0},{"id":"ult","name":"Overdrive","icon":0,"glyph":"star","key":"Z","cooldown":0,"time":"","count":0,"ready":false,"disabled":false,"ultimate":true,"charge":0.74},{"id":"stim","name":"Stim","icon":0,"glyph":"potion","key":"4","cooldown":0,"time":"","count":4,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"wall","name":"Barrier","icon":0,"glyph":"shield","key":"X","cooldown":0.92,"time":"38","count":0,"ready":false,"disabled":true,"ultimate":false,"charge":0}]}'>
<meta name="fixture:ready" content='{"slots":[{"id":"q","name":"Phase dash","icon":0,"glyph":"bolt","key":"Q","cooldown":0,"time":"","count":0,"ready":true,"disabled":false,"ultimate":false,"charge":0},{"id":"ult","name":"Overdrive","icon":0,"glyph":"star","key":"Z","cooldown":0,"time":"","count":0,"ready":true,"disabled":false,"ultimate":true,"charge":1}]}'>
<meta name="fixture:cooling" content='{"slots":[{"id":"q","name":"Phase dash","icon":0,"glyph":"bolt","key":"Q","cooldown":1,"time":"12","count":0,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"e","name":"Scan","icon":0,"glyph":"map","key":"E","cooldown":0.1,"time":"1","count":0,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"ult","name":"Overdrive","icon":0,"glyph":"star","key":"Z","cooldown":0,"time":"","count":0,"ready":false,"disabled":false,"ultimate":true,"charge":0}]}'>
<meta name="fixture:empty" content='{"slots":[]}'>
<meta name="fixture:overflow" content='{"slots":[{"id":"a","name":"A","icon":0,"glyph":"bolt","key":"Left Shoulder","cooldown":0.5,"time":"1:02:44","count":0,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"b","name":"B","icon":0,"glyph":"bomb","key":"Right Trigger","cooldown":0,"time":"","count":12345,"ready":true,"disabled":false,"ultimate":false,"charge":0},{"id":"ult","name":"C","icon":0,"glyph":"star","key":"Left Stick + Right Stick","cooldown":0,"time":"","count":0,"ready":false,"disabled":false,"ultimate":true,"charge":0.999},{"id":"d","name":"D","icon":0,"glyph":"potion","key":"Dpad Up","cooldown":0,"time":"","count":99,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"e","name":"E","icon":0,"glyph":"shield","key":"Y","cooldown":0.2,"time":"8","count":0,"ready":false,"disabled":true,"ultimate":false,"charge":0},{"id":"f","name":"F","icon":0,"glyph":"map","key":"B","cooldown":0,"time":"","count":0,"ready":false,"disabled":false,"ultimate":false,"charge":0}]}'>

<!--
  The ability bar at the foot of the screen, built from HudSlot: each
  ability in an octagon with a lit edge, its cooldown a dark sweep with a
  lit leading edge and the seconds left, its charges on a tag and its key
  cap under it; the ultimate in a larger hexagon inside a ring in the
  HUD's accent that fills with its charge. The game owns every cooldown
  and charge; read only.
-->
<ul id="ShooterAbilities" part="ShooterAbilities.frame" class="hud bare" data-layout="row" aria-label={t("shooter.abilities.name")}>
  {#each slots as s (s.id)}
  <li part="ShooterAbilities.ability" class="ability" data-ultimate={s.ultimate}><HudSlot name={s.name} icon={s.icon} glyph={s.glyph} key={s.key} kind={s.ultimate ? 'ultimate' : s.count > 0 ? 'item' : 'skill'} cooldown={s.ultimate ? 0 : s.cooldown} time={s.time} count={s.count} ready={s.ready} unaffordable={s.disabled} reason="" charges={0} chargesMax={0} charge={s.charge} /></li>
  {/each}
</ul>

<style>
:host { margin: 0; padding: 0; list-style: none; align-items: flex-end; column-gap: calc(14 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); --hud-slot-ring: var(--hud-accent); }
/* The slots' places: the ultimate stands larger. */
.ability { display: flex; flex: 0 1 auto; min-inline-size: 0; --hud-slot-size: calc(52 * var(--font-step-px)); }
.ability[data-ultimate=true] { --hud-slot-size: calc(84 * var(--font-step-px)); }
@media (width < 600px), (height < 500px) {
  :host { column-gap: calc(8 * var(--font-step-px)); }
  .ability { --hud-slot-size: calc(36 * var(--font-step-px)); }
  .ability[data-ultimate=true] { --hud-slot-size: calc(46 * var(--font-step-px)); }
}
</style>
