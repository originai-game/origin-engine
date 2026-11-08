<meta name="component" content="RadialMenu">
<meta name="strings" content="./radial-menu.en.ogt">
<meta name="prop:items" content="list" data-doc="The slices, clockwise from the top: each { id, label, icon, at, cost, unaffordable, disabled }, at its centre as a turn fraction (slice i of n: i / n).">
<meta name="prop:span" content="number" data-doc="One slice's share of the turn (1 / n).">
<meta name="prop:chosen" content="string" data-doc="The slice the pointer or stick points at (its id); empty is none.">
<meta name="prop:detail" content="string" data-doc="The centre's line for the chosen slice; empty shows its label.">
<meta name="action:aim" content="Aim">
<meta name="action:choose" content="Choose">
<meta name="fixture:default" content='{"span":0.1667,"chosen":"heal","detail":"Restore 40 health","items":[{"id":"heal","label":"Heal","icon":"heart","at":0,"cost":"","unaffordable":false,"disabled":false},{"id":"shield","label":"Ward","icon":"shield","at":0.1667,"cost":"150","unaffordable":false,"disabled":false},{"id":"bomb","label":"Bomb","icon":"bomb","at":0.3333,"cost":"300","unaffordable":true,"disabled":false},{"id":"map","label":"Map","icon":"map","at":0.5,"cost":"","unaffordable":false,"disabled":false},{"id":"bag","label":"Bag","icon":"bag","at":0.6667,"cost":"","unaffordable":false,"disabled":true},{"id":"flag","label":"Rally","icon":"flag","at":0.8333,"cost":"","unaffordable":false,"disabled":false}]}'>

<div id="RadialMenu" role="menu" aria-label={t("component.radial-menu.name")} style:--span={span} onpointermove={aim}>
  <svg id="Disc" viewBox="0 0 320 320"><circle cx="160" cy="160" r="150" fill="#0a0d14b8"></circle></svg>
  {#each items as item (item.id)}
    <div class="slice" class:chosen={item.id == chosen} class:disabled={item.disabled} style:--at={item.at}>
      <svg class="wedge" viewBox="0 0 320 320"><circle cx="160" cy="160" r="105" fill="none" stroke-width="86" pathLength="1" stroke-dasharray={(span - 0.006) + " 1"} transform={"rotate(" + ((item.at - span / 2) * 360 - 90 + 1) + " 160 160)"}></circle></svg>
      <button class="item" role="menuitem" aria-label={item.label} disabled={item.disabled} onclick={choose}>
        <div class="upright">
          <icon class="icon" icon={item.icon}></icon>
          <span class="label">{item.label}</span>
          <div class="cost" class:short={item.unaffordable} hidden={!item.cost}><icon class="coin" icon="coin"></icon><span class="price">{item.cost}</span></div>
        </div>
      </button>
    </div>
  {/each}
  <div id="Centre">
    <span id="Detail">{detail}</span>
  </div>
</div>

<style>
#RadialMenu { width: 320px; height: 320px; }
#Disc { position: absolute; top: 0; inset-inline-start: 0; width: 320px; height: 320px; }
.slice { position: absolute; top: 0; inset-inline-start: 0; width: 320px; height: 320px; }
.wedge { position: absolute; top: 0; inset-inline-start: 0; width: 320px; height: 320px; stroke: #ffffff14; }
.slice.chosen .wedge { stroke: var(--primary); }
.item { position: absolute; top: 18px; inset-inline-start: 120px; width: 80px; height: 84px; padding: 0; background: #00000000; border: none; rotate: calc(var(--at) * 360deg); transform-origin: 50% 169.05%; }
.upright { width: stretch; height: stretch; align-items: center; justify-content: center; gap: 2px; rotate: calc(var(--at) * -360deg); }
.icon { width: 28px; height: 28px; color: var(--ink); }
.label { font: var(--text-caption); color: var(--ink); white-space: nowrap; }
.cost { display: flex; gap: 2px; align-items: center; color: var(--gold); }
.price { font: var(--text-caption); color: var(--gold); }
.cost.short .price { color: var(--danger); }
.cost.short { color: var(--danger); }
.coin { width: 12px; height: 12px; }
.slice.disabled .item { opacity: 0.4; }
.slice.chosen .icon { scale: 1.15; color: var(--on-primary); }
.slice.chosen .label { color: var(--on-primary); }
#Centre { position: absolute; top: 112px; inset-inline-start: 112px; width: 96px; height: 96px; border-radius: 48px; background: var(--panel-raised); border: 1px solid var(--line-strong); align-items: center; justify-content: center; padding: 8px; }
#Detail { font: var(--text-caption); color: var(--ink); text-align: center; }
</style>
