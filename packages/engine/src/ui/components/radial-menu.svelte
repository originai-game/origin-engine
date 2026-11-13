<meta name="component" content="RadialMenu">
<meta name="strings" content="./radial-menu.en.ogt">
<meta name="parts" content="RadialMenu.disc RadialMenu.slice RadialMenu.label RadialMenu.centre">
<meta name="prop:items" content="list" data-doc="The slices, clockwise from the top: each { id, label, icon, at, cost, unaffordable, disabled }, at its centre as a turn fraction (slice i of n: i / n).">
<meta name="prop:span" content="number" data-doc="One slice's share of the turn (1 / n).">
<meta name="prop:chosen" content="string" data-doc="The slice the pointer or stick points at (its id); empty is none.">
<meta name="prop:detail" content="string" data-doc="The centre's line for the chosen slice; empty shows its label.">
<meta name="action:aim" content="Aim">
<meta name="action:choose" content="Choose">
<meta name="fixture:default" content='{"span":0.1667,"chosen":"heal","detail":"Restore 40 health","items":[{"id":"heal","label":"Heal","icon":"heart","at":0,"cost":"","unaffordable":false,"disabled":false},{"id":"shield","label":"Ward","icon":"shield","at":0.1667,"cost":"150","unaffordable":false,"disabled":false},{"id":"bomb","label":"Bomb","icon":"bomb","at":0.3333,"cost":"300","unaffordable":true,"disabled":false},{"id":"map","label":"Map","icon":"map","at":0.5,"cost":"","unaffordable":false,"disabled":false},{"id":"bag","label":"Bag","icon":"bag","at":0.6667,"cost":"","unaffordable":false,"disabled":true},{"id":"flag","label":"Rally","icon":"flag","at":0.8333,"cost":"","unaffordable":false,"disabled":false}]}'>

<div id="RadialMenu" role="menu" aria-label={t("component.radial-menu.name")} style:--span={span} onpointermove={aim}>
  <svg id="Disc" part="RadialMenu.disc" viewBox="0 0 320 320"><circle cx="160" cy="160" r="150"></circle></svg>
  {#each items as item (item.id)}
    <div class="slice" part="RadialMenu.slice" class:chosen={item.id == chosen} class:disabled={item.disabled} style:--at={item.at}>
      <svg class="wedge" viewBox="0 0 320 320"><circle cx="160" cy="160" r="105" fill="none" stroke-width="86" pathLength="1" stroke-dasharray={(span - 0.006) + " 1"} transform={"rotate(" + ((item.at - span / 2) * 360 - 90 + 1) + " 160 160)"}></circle></svg>
      <button class="item" role="menuitem" aria-label={item.label} disabled={item.disabled} onclick={choose}>
        <div class="upright">
          <og-icon class="icon" icon={item.icon}></og-icon>
          <span class="label" part="RadialMenu.label">{item.label}</span>
          <div class="cost" class:short={item.unaffordable} hidden={!item.cost}><og-icon class="coin" icon="coin"></og-icon><span class="price">{item.cost}</span></div>
        </div>
      </button>
    </div>
  {/each}
  <div id="Centre" part="RadialMenu.centre">
    <span id="Detail">{detail}</span>
  </div>
</div>

<style>
/* The wheel is two --size-4xl across; every offset below is a share of that. */
#RadialMenu { --wheel: calc(var(--size-4xl) * 2); width: var(--wheel); height: var(--wheel); }
[part="RadialMenu.disc"] { position: absolute; top: 0; inset-inline-start: 0; width: var(--wheel); height: var(--wheel); fill: var(--hud); }
[part="RadialMenu.slice"] { position: absolute; top: 0; inset-inline-start: 0; width: var(--wheel); height: var(--wheel); }
.wedge { position: absolute; top: 0; inset-inline-start: 0; width: var(--wheel); height: var(--wheel); stroke: var(--sheen); }
.chosen .wedge { stroke: var(--primary); }
.item { position: absolute; top: var(--space-lg); inset-inline-start: calc(var(--size-4xl) * 0.75); width: calc(var(--size-4xl) / 2); height: calc(var(--size-4xl) / 2); padding: 0; background: transparent; border: none; rotate: calc(var(--at) * 360deg); transform-origin: 50% 180%; }
.upright { width: stretch; height: stretch; align-items: center; justify-content: center; gap: var(--space-xxs); rotate: calc(var(--at) * -360deg); }
.icon { width: var(--size-sm); height: var(--size-sm); color: var(--ink); }
[part="RadialMenu.label"] { font: var(--text-caption); font-family: var(--label-font); white-space: nowrap; }
.cost { display: flex; gap: var(--space-xxs); align-items: center; color: var(--gold); }
.price { font: var(--text-caption); color: var(--gold); }
.cost.short .price { color: var(--danger); }
.cost.short { color: var(--danger); }
.coin { width: var(--size-xs); height: var(--size-xs); }
.disabled .item { opacity: var(--opacity-dim); }
.chosen .icon { scale: var(--motion-scale-swell); color: var(--on-primary); }
.chosen [part="RadialMenu.label"] { color: var(--on-primary); }
[part="RadialMenu.centre"] { position: absolute; top: calc(var(--size-4xl) - var(--size-xxl) / 2); inset-inline-start: calc(var(--size-4xl) - var(--size-xxl) / 2); width: var(--size-xxl); height: var(--size-xxl); border-radius: var(--radius-capsule); background: var(--panel-raised); border: var(--stroke-hairline) solid var(--line-strong); align-items: center; justify-content: center; padding: var(--space-sm); }
#Detail { font: var(--text-caption); color: var(--ink); text-align: center; }
</style>
