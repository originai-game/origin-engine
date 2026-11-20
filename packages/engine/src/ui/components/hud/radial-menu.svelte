<meta name="component" content="RadialMenu">
<meta name="strings" content="./radial-menu.en.ogt">
<meta name="parts" content="RadialMenu.disc RadialMenu.slice RadialMenu.item RadialMenu.label RadialMenu.centre RadialMenu.cell RadialMenu.well RadialMenu.caption RadialMenu.figure">
<meta name="prop:items" content="list" data-doc="The slices, clockwise from the top: each { id, label, icon, at, cost, unaffordable, disabled }, at its centre as a turn fraction (slice i of n: i / n).">
<meta name="prop:span" content="number" data-doc="One slice's share of the turn (1 / n).">
<meta name="prop:chosen" content="string" data-doc="The slice the pointer or stick points at (its id); empty is none.">
<meta name="prop:detail" content="string" data-doc="The centre's line for the chosen slice; empty shows its label.">
<meta name="action:aim" content="Aim">
<meta name="action:choose" content="Choose">
<meta name="fixture:default" content='{"span":0.1667,"chosen":"heal","detail":"Restore 40 health","items":[{"id":"heal","label":"Heal","icon":"heart","at":0,"cost":"","unaffordable":false,"disabled":false},{"id":"shield","label":"Ward","icon":"shield","at":0.1667,"cost":"150","unaffordable":false,"disabled":false},{"id":"bomb","label":"Bomb","icon":"bomb","at":0.3333,"cost":"300","unaffordable":true,"disabled":false},{"id":"map","label":"Map","icon":"map","at":0.5,"cost":"","unaffordable":false,"disabled":false},{"id":"bag","label":"Bag","icon":"bag","at":0.6667,"cost":"","unaffordable":false,"disabled":true},{"id":"flag","label":"Rally","icon":"flag","at":0.8333,"cost":"","unaffordable":false,"disabled":false}]}'>

<div id="RadialMenu" data-layout="overlay" role="menu" aria-label={t("component.radial-menu.name")} style:--span={span} onpointermove={aim}>
  <svg id="Disc" part="RadialMenu.disc" data-anchor="fill" viewBox="0 0 320 320"><circle cx="160" cy="160" r="150"></circle></svg>
  {#each items as item (item.id)}
    <div class="slice" part="RadialMenu.slice" data-layout="overlay" data-anchor="fill" class:chosen={item.id == chosen} class:disabled={item.disabled} style:--at={item.at}>
      <svg class="wedge" data-anchor="fill" viewBox="0 0 320 320"><circle cx="160" cy="160" r="105" fill="none" stroke-width="86" pathLength="1" stroke-dasharray={(span - 0.006) + " 1"} transform={"rotate(" + ((item.at - span / 2) * 360 - 90 + 1) + " 160 160)"}></circle></svg>
      <button part="RadialMenu.item RadialMenu.cell" role="menuitem" aria-label={item.label} aria-current={item.id == chosen} disabled={item.disabled} onclick={choose}>
        <div class="upright" data-layout="stack" data-gap="hair">
          <og-icon class="icon" icon={item.icon}></og-icon>
          <span class="label" part="RadialMenu.label">{item.label}</span>
          {#if item.cost}<div class="cost" data-layout="row" data-gap="hair" data-tone={item.unaffordable ? 'danger' : 'warn'} class:short={item.unaffordable}><og-icon class="coin" icon="coin"></og-icon><span class="price" part="RadialMenu.figure" data-tone={item.unaffordable ? 'danger' : 'warn'}>{item.cost}</span></div>{/if}
        </div>
      </button>
    </div>
  {/each}
  <div id="Centre" part="RadialMenu.centre RadialMenu.well" data-layout="cover" data-gap="none" data-anchor="center">
    {#if detail}<span id="Detail" class="detail" part="RadialMenu.caption">{detail}</span>
    {:else}{#each items as item (item.id)}{#if item.id == chosen}<span class="detail" part="RadialMenu.caption">{item.label}</span>{/if}{/each}{/if}
  </div>
</div>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
/* The wheel is two --size-4xl across; every offset below is a share of that. */
:host { --wheel: calc(var(--size-4xl) * 2); width: var(--wheel); height: var(--wheel); --layout-anchor-inset: 0; }
[part="RadialMenu.disc"] { width: var(--wheel); height: var(--wheel); fill: var(--hud); }
[part="RadialMenu.slice"] { width: var(--wheel); height: var(--wheel); }
.wedge { width: var(--wheel); height: var(--wheel); stroke: var(--cell-rim); }
.chosen .wedge { stroke: var(--cell-selected-rim); }
/* The wedge is the slice's plate: its button draws none. */
[part="RadialMenu.item RadialMenu.cell"] { position: absolute; top: var(--space-lg); inset-inline-start: calc(var(--size-4xl) * 0.75); width: calc(var(--size-4xl) / 2); height: calc(var(--size-4xl) / 2); padding: 0; background: none; border: none; box-shadow: none; rotate: calc(var(--at) * 360deg); transform-origin: 50% 180%; }
.upright { inline-size: 100%; block-size: 100%; align-items: center; justify-content: center; rotate: calc(var(--at) * -360deg); }
.icon { width: var(--size-sm); height: var(--size-sm); }
[part="RadialMenu.label"] { white-space: nowrap; }
.price { --figure-size: var(--font-step-sm); }
.coin { width: var(--size-xs); height: var(--size-xs); }
.chosen [part~="RadialMenu.cell"] { --ink: var(--on-primary); }
.chosen .icon { scale: var(--motion-scale-swell); }
.chosen [part="RadialMenu.label"] { --ink-label: var(--on-primary); }
[part="RadialMenu.centre RadialMenu.well"] { width: var(--size-xxl); height: var(--size-xxl); --well-radius: var(--radius-capsule); padding: var(--space-sm); }
#Detail, .detail { text-align: center; pointer-events: none; }
</style>
