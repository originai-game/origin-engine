<meta name="component" content="ShooterPickups">
<meta name="strings" content="./shooter.en.ogt">
<meta name="parts" content="ShooterPickups.frame ShooterPickups.entry ShooterPickups.art ShooterPickups.item ShooterPickups.count">
<meta name="prop:items" content="list" data-doc="What the player just picked up, newest first, keyed rows of { id, name, icon, glyph, quantity, rarity }: icon a texture (0: the glyph, an icon name, in its place); quantity the count gained (+30); rarity common, rare, epic or legendary (the edge's and the count's colour, on data-rarity). The game trims old rows (a phone shows the newest); a new row slides in from the screen's edge and an old one fades out.">
<meta name="fixture:default" content='{"items":[{"id":"frag","name":"Frag grenade","icon":0,"glyph":"bomb","quantity":1,"rarity":"legendary"},{"id":"cell","name":"Shield cell","icon":0,"glyph":"shield","quantity":2,"rarity":"rare"},{"id":"light","name":"Light rounds","icon":0,"glyph":"bolt","quantity":30,"rarity":"common"}]}'>
<meta name="fixture:empty" content='{"items":[]}'>
<meta name="fixture:epic" content='{"items":[{"id":"optic","name":"Holo optic","icon":0,"glyph":"gem","quantity":1,"rarity":"epic"}]}'>
<meta name="fixture:overflow" content='{"items":[{"id":"long","name":"Extended heavy magazine for the Northern Relay marksman rifle","icon":0,"glyph":"","quantity":123456,"rarity":"legendary"}]}'>

<!--
  The pickup feed lower on the screen's side: each row an icon, the item's
  name and the count gained on a dark glass that fades toward the middle,
  its edge at the screen's side tinted by rarity. Read only.
-->
<section id="ShooterPickups" part="ShooterPickups.frame" class="hud bare" data-layout="stack" role="log" aria-label={t("shooter.pickups.name")}>
  {#each items as p (p.id)}
  <div part="ShooterPickups.entry" class="entry" data-layout="row" data-gap="tight" data-rarity={p.rarity} aria-label={t("shooter.pickups.row", { name: p.name, quantity: p.quantity })} in:fly={{ x: 24, duration: 200 }} out:fade={{ duration: 300 }} animate:flip={{ duration: 200 }}>
    <div part="ShooterPickups.art" class="art" data-layout="overlay">
      {#if p.icon}<img class="picture" data-anchor="fill" src={p.icon} alt="">{:else}{#if p.glyph}<og-icon class="glyph" data-anchor="fill" icon={p.glyph}></og-icon>{/if}{/if}
    </div>
    <span part="ShooterPickups.item" class="item">{p.name}</span>
    <span part="ShooterPickups.count" class="count">{t("shooter.pickups.count", { quantity: formatNumber(p.quantity) })}</span>
    <div class="edge"></div>
  </div>
  {/each}
</section>

<style>
:host { align-items: flex-end; row-gap: calc(5 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); --layout-anchor-inset: 0; }
.entry { flex: none; align-items: center; max-inline-size: 100%; min-inline-size: 0; padding-inline-start: var(--space-sm); background: var(--hud-feed-fill); }
.entry:dir(rtl) { background: var(--hud-feed-fill-start); }
/* The edge at the screen's side, tinted by rarity. */
.edge { flex: none; align-self: stretch; inline-size: var(--hud-feed-edge-width); margin-inline-start: var(--space-xs); background: var(--hud-feed-edge); }
[data-rarity=rare] .edge { background: var(--hud-rare); }
[data-rarity=epic] .edge { background: var(--hud-epic); }
[data-rarity=legendary] .edge { background: var(--hud-legendary); }
.art { flex: none; margin-block: calc(2 * var(--font-step-px)); inline-size: calc(22 * var(--font-step-px)); block-size: calc(22 * var(--font-step-px)); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.picture { object-fit: cover; }
.glyph { color: var(--hud-ink); }
.item { min-inline-size: 0; font-size: var(--hud-size-feed); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.count { flex: none; font-size: var(--hud-size-feed); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; color: var(--hud-ink-dim); }
[data-rarity=rare] .count { color: var(--hud-rare); }
[data-rarity=epic] .count { color: var(--hud-epic); }
[data-rarity=legendary] .count { color: var(--hud-legendary); }
@media (width < 600px) {
  .entry:nth-child(n + 2) { display: none; }
  .art { inline-size: calc(18 * var(--font-step-px)); block-size: calc(18 * var(--font-step-px)); }
  .item, .count { font-size: var(--hud-size-xs); }
}
</style>
