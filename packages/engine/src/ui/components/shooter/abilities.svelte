<meta name="component" content="ShooterAbilities">
<meta name="strings" content="./shooter.en.ogt">
<meta name="prop:slots" content="list" data-doc="The abilities and items on the bar in the game's order, keyed rows of { id, name, icon, glyph, key, cooldown, time, count, ready, disabled, ultimate, charge }: name for a reader; icon a texture (0: the glyph, an icon name, in its place); key the input as the game names it (Q), on a key cap under the slot; cooldown the share of its cooldown still to run, 0 to 1 (a dark sweep over the art), with time its figure as the game words it (6); count the charges or items held, on a tag (×2; 0 hides it); ready lit (a lit edge and glow); disabled greyed. The ultimate (ultimate true) stands larger in a hexagon inside a ring that fills with its charge, 0 to 1, its share in figures until it is ready.">
<meta name="fixture:default" content='{"slots":[{"id":"dash","name":"Phase dash","icon":0,"glyph":"bolt","key":"Q","cooldown":0.38,"time":"6","count":0,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"frag","name":"Frag grenade","icon":0,"glyph":"bomb","key":"G","cooldown":0,"time":"","count":2,"ready":true,"disabled":false,"ultimate":false,"charge":0},{"id":"ult","name":"Overdrive","icon":0,"glyph":"star","key":"Z","cooldown":0,"time":"","count":0,"ready":false,"disabled":false,"ultimate":true,"charge":0.74},{"id":"stim","name":"Stim","icon":0,"glyph":"potion","key":"4","cooldown":0,"time":"","count":4,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"wall","name":"Barrier","icon":0,"glyph":"shield","key":"X","cooldown":0.92,"time":"38","count":0,"ready":false,"disabled":true,"ultimate":false,"charge":0}]}'>
<meta name="fixture:ready" content='{"slots":[{"id":"q","name":"Phase dash","icon":0,"glyph":"bolt","key":"Q","cooldown":0,"time":"","count":0,"ready":true,"disabled":false,"ultimate":false,"charge":0},{"id":"ult","name":"Overdrive","icon":0,"glyph":"star","key":"Z","cooldown":0,"time":"","count":0,"ready":true,"disabled":false,"ultimate":true,"charge":1}]}'>
<meta name="fixture:cooling" content='{"slots":[{"id":"q","name":"Phase dash","icon":0,"glyph":"bolt","key":"Q","cooldown":1,"time":"12","count":0,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"e","name":"Scan","icon":0,"glyph":"map","key":"E","cooldown":0.1,"time":"1","count":0,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"ult","name":"Overdrive","icon":0,"glyph":"star","key":"Z","cooldown":0,"time":"","count":0,"ready":false,"disabled":false,"ultimate":true,"charge":0}]}'>
<meta name="fixture:empty" content='{"slots":[]}'>
<meta name="fixture:overflow" content='{"slots":[{"id":"a","name":"A","icon":0,"glyph":"bolt","key":"Left Shoulder","cooldown":0.5,"time":"1:02:44","count":0,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"b","name":"B","icon":0,"glyph":"bomb","key":"Right Trigger","cooldown":0,"time":"","count":12345,"ready":true,"disabled":false,"ultimate":false,"charge":0},{"id":"ult","name":"C","icon":0,"glyph":"star","key":"Left Stick + Right Stick","cooldown":0,"time":"","count":0,"ready":false,"disabled":false,"ultimate":true,"charge":0.999},{"id":"d","name":"D","icon":0,"glyph":"potion","key":"Dpad Up","cooldown":0,"time":"","count":99,"ready":false,"disabled":false,"ultimate":false,"charge":0},{"id":"e","name":"E","icon":0,"glyph":"shield","key":"Y","cooldown":0.2,"time":"8","count":0,"ready":false,"disabled":true,"ultimate":false,"charge":0},{"id":"f","name":"F","icon":0,"glyph":"map","key":"B","cooldown":0,"time":"","count":0,"ready":false,"disabled":false,"ultimate":false,"charge":0}]}'>

<!--
  The ability bar at the foot of the screen: each ability in an octagon
  with a lit edge, its cooldown a dark sweep over the art with the seconds
  left, its charges on a tag and its key cap under it; the ultimate in a
  larger hexagon inside a ring that fills with its charge. The game owns
  every cooldown and charge; read only.
-->
<ul id="ShooterAbilities" part="frame" variant="hud" layout="row" aria-label={t("shooter.abilities.name")}>
  {#each slots as s (s.id)}
  <li part="ability" class="ability" layout="stack" gap="none" aria-label={s.name} class:ultimate={s.ultimate} class:ready={s.ready || (s.ultimate && s.charge >= 1)} class:disabled={s.disabled}>
    <div class="holder" layout="overlay">
      {#if s.ultimate}<div part="ring" class="ring" anchor="fill" role="meter" aria-label={t("shooter.abilities.charge")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={s.charge <= 0 ? 0 : s.charge >= 1 ? 100 : s.charge * 100}><svg class="dial" viewBox="0 0 100 100"><path class="well" d="M 50 4 A 46 46 0 1 1 50 96 A 46 46 0 1 1 50 4" fill="none"></path><path class="lit" d="M 50 4 A 46 46 0 1 1 50 96 A 46 46 0 1 1 50 4" fill="none" pathLength="1" stroke-dasharray={(s.charge <= 0 ? 0 : s.charge >= 1 ? 1 : s.charge) + " 1"}></path></svg></div>{/if}
      <div part="slot" class="slot" anchor="fill" layout="overlay">
        {#if s.icon}<img part="art" class="art" anchor="fill" src={s.icon} alt="">{:else}<div class="blank" anchor="fill" layout="center">{#if s.glyph}<og-icon class="glyph" icon={s.glyph}></og-icon>{/if}</div>{/if}
        {#if s.cooldown > 0}<div part="cooldown" class="cooldown" anchor="fill" style:--left={s.cooldown >= 1 ? 1 : s.cooldown}></div>{/if}
        {#if s.cooldown > 0 && s.time}<span layout="center" part="time" class="time" anchor="fill">{s.time}</span>{/if}
        {#if s.ultimate && s.charge < 1}<span layout="center" part="charge" class="time" anchor="fill">{formatPercent(s.charge <= 0 ? 0 : s.charge)}</span>{/if}
      </div>
      {#if s.count > 0}<span part="count" class="tag" anchor="top-end">{t("shooter.abilities.count", { count: formatNumber(s.count) })}</span>{/if}
    </div>
    {#if s.key}<span part="key" class="key">{s.key}</span>{/if}
  </li>
  {/each}
</ul>

<style>
#ShooterAbilities { margin: 0; padding: 0; list-style: none; align-items: flex-end; column-gap: calc(14 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); --s: calc(52 * var(--font-step-px)); --ult: calc(84 * var(--font-step-px)); }
.ability { flex: 0 1 auto; min-inline-size: 0; align-items: center; row-gap: calc(5 * var(--font-step-px)); filter: var(--hud-lift); }
.holder { inline-size: var(--s); block-size: var(--s); --layout-anchor-inset: 0; }
.ultimate .holder { inline-size: var(--ult); block-size: var(--ult); }
/* The slot: an octagon whose fill is its lit edge, the art inset inside. */
.slot { inline-size: var(--s); block-size: var(--s); clip-path: var(--hud-octagon); background: var(--hud-edge); --layout-anchor-inset: calc(2 * var(--font-step-px)); }
.ultimate .slot { inline-size: calc(var(--ult) * 0.78); block-size: calc(var(--ult) * 0.78); clip-path: var(--hud-hexagon); }
.ready .slot { background: var(--hud-ready-edge); }
.ready .holder { filter: var(--hud-ready-glow); }
.art, .blank, .cooldown { clip-path: var(--hud-octagon); }
.ultimate .art, .ultimate .blank, .ultimate .cooldown { clip-path: var(--hud-hexagon); }
.art { object-fit: cover; }
.blank { background: var(--hud-plate-fill); }
.glyph { inline-size: 50%; block-size: 50%; color: var(--hud-ink); }
.disabled .art, .disabled .glyph { filter: var(--hud-unready); }
/* The cooldown still to run is dark; it turns clockwise from the top. */
.cooldown { background: var(--hud-sweep); mask-image: conic-gradient(transparent calc((1 - var(--left)) * 360deg), #000 0); }
.time { font-size: var(--hud-size-md); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.ultimate .time { font-size: var(--hud-size-sm); }
/* The ultimate's ring: a well and the charge as a lit arc. */
.dial { inline-size: 100%; block-size: 100%; }
.well, .lit { stroke-width: 7; }
.well { stroke: var(--hud-ring-well); }
.lit { stroke: var(--hud-accent); }
.tag { max-inline-size: calc(var(--s) * 1.2); margin-block-start: calc(-6 * var(--font-step-px)); margin-inline-end: calc(-6 * var(--font-step-px)); padding: 0 calc(5 * var(--font-step-px)); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.key { min-inline-size: calc(22 * var(--font-step-px)); max-inline-size: calc(var(--s) * 1.3); padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); text-align: center; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
/* Under a touch screen the slots are the controls: no key caps. */
.key:touch { display: none; }
#ShooterAbilities:size(compact) { column-gap: calc(8 * var(--font-step-px)); --s: calc(36 * var(--font-step-px)); --ult: calc(44 * var(--font-step-px)); }
.time:size(compact) { font-size: var(--hud-size-sm); }
.ultimate .time:size(compact) { font-size: var(--hud-size-xs); }
</style>
