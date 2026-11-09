<meta name="component" content="Loadout">
<meta name="strings" content="./shooter.en.ogt">
<meta name="prop:title" content="string" data-doc="The page's heading (Loadout).">
<meta name="prop:categories" content="list" data-doc="The slot groups in order, a keyed collection of { id, label } (primary, secondary, melee, utility).">
<meta name="prop:slots" content="list" data-doc="What can be carried, a keyed collection of { id, category, name, icon, detail, equipped, locked, cost }: category a category's id; icon an icon of the style's atlas; detail a short line under the name (ammunition, damage); equipped it is carried now; locked it cannot be chosen yet (shown with its cost); cost text, empty for none.">
<meta name="prop:selected" content="string" data-doc="The slot id selected now (empty: none).">
<meta name="action:details" content="Equip">
<meta name="action:back" content="Back">
<meta name="parts" content="Loadout.item Loadout.category">
<meta name="fixture:default" content='{"title":"Loadout","selected":"strider","categories":[{"id":"primary","label":"Primary"},{"id":"secondary","label":"Secondary"},{"id":"melee","label":"Melee"}],"slots":[{"id":"strider","category":"primary","name":"Strider AR","icon":"sword","detail":"25 / 75","equipped":true,"locked":false,"cost":"2,900"},{"id":"halcyon","category":"primary","name":"Halcyon","icon":"sword","detail":"30 / 90","equipped":false,"locked":false,"cost":"2,900"},{"id":"pike","category":"primary","name":"Pike .50","icon":"sword","detail":"5 / 10","equipped":false,"locked":true,"cost":"4,700"},{"id":"wren","category":"secondary","name":"Wren Pistol","icon":"bolt","detail":"15 / 45","equipped":true,"locked":false,"cost":"500"},{"id":"knife","category":"melee","name":"Field Knife","icon":"sword","detail":"","equipped":true,"locked":false,"cost":""}]}'>
<meta name="fixture:empty" content='{"title":"Loadout","selected":"","categories":[{"id":"primary","label":"Primary"}],"slots":[]}'>

<section id="Loadout" aria-label={title}>
  <h2 id="Title">{title}</h2>
  <div id="Groups">
    {#each categories as c (c.id)}
      <section class="group" aria-label={c.label}>
        <h3 class="category" part="Loadout.category">{c.label}</h3>
        <div class="items">
          {#each slots as s (s.id)}
            {#if s.category == c.id}
              <button class="item" part="Loadout.item" aria-selected={s.id == selected} aria-label={s.name} class:equipped={s.equipped} class:locked={s.locked} disabled={s.locked} onclick={select} ondetails={equip}><div class="face">
                <icon class="icon" icon={s.locked ? 'lock' : s.icon}></icon>
                <div class="text">
                  <span class="name">{s.name}</span>
                  <div class="sub" hidden={s.detail == '' && !s.equipped}>
                    <span class="detail" hidden={s.detail == ''}>{s.detail}</span>
                    <span class="badge" hidden={!s.equipped}>{t("shooter.loadout.equipped")}</span>
                  </div>
                </div>
                {#if s.cost != ''}<div class="cost"><icon class="coin" icon="coin"></icon><span class="price">{s.cost}</span></div>{/if}
              </div></button>
            {/if}
          {/each}
        </div>
      </section>
    {/each}
    {#if slots.length == 0}
      <p id="Empty">{t("shooter.loadout.empty")}</p>
    {/if}
    <div id="Fade"></div>
  </div>
  <footer id="Actions">
    <button id="Back" variant="ghost" onclick={back} onback={back}><kbd id="BackKey" data-action="back"></kbd><span id="BackLabel">{t("shooter.loadout.back")}</span></button>
    {#each slots as s (s.id)}
      {#if s.id == selected}
        <button id="Equip" variant="primary" disabled={s.locked || s.equipped} onclick={equip}><kbd id="EquipKey" data-action="details"></kbd><span id="EquipLabel">{t("shooter.loadout.equip")}</span></button>
      {/if}
    {/each}
  </footer>
</section>

<style>
#Loadout { width: stretch; height: stretch; gap: var(--space-lg); }
#Title { font: var(--text-xl2); font-family: var(--font-display); }
#Groups { width: stretch; height: stretch; overflow-y: auto; scrollbar-gutter: stable; padding-inline-end: var(--space-lg); container-type: scroll-state; gap: var(--space-xl); display: grid; grid-template-columns: repeat(auto-fill, minmax(calc(var(--size-xxl) * 3), 1fr)); align-content: start; }
.group { gap: var(--space-sm); }
.category { font: var(--text-caption); color: var(--ink-faint); text-transform: uppercase; letter-spacing: 0.08em; }
.items { gap: var(--space-xs); }
.item { width: stretch; height: auto; min-height: var(--size-touch); padding: var(--space-sm) var(--space-md); border-radius: var(--radius-sm); }
.face { display: flex; width: stretch; align-items: center; gap: var(--space-md); }
.icon { width: var(--size-sm); height: var(--size-sm); color: var(--ink-muted); }
.text { width: stretch; gap: var(--space-xxs); align-items: flex-start; }
.name { font: var(--text-body); text-align: start; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: stretch; }
.sub { display: flex; align-items: center; gap: var(--space-sm); }
.detail { text-align: start; font: var(--text-caption); font-family: var(--font-mono); color: var(--ink-muted); }
/* Equipped is a tag on the detail line, so every row's price stands alone at its end
   and the name keeps the row's width. */
.badge { flex: none; font: var(--text-caption); color: var(--accent); text-transform: uppercase; white-space: nowrap; padding: 0 var(--space-xs); border: 1px solid var(--accent); border-radius: var(--radius-xs); }
.cost { display: flex; flex: none; align-items: center; gap: var(--space-xxs); }
.price { font: var(--text-caption); font-family: var(--font-mono); color: var(--ink-muted); white-space: nowrap; }
.coin { width: var(--size-xs); height: var(--size-xs); color: var(--gold); }
.locked .name, .locked .icon { color: var(--ink-faint); }
#Empty { font: var(--text-body); color: var(--ink-muted); }
/* The bottom edge fades while there is more to scroll, so a cut row reads as more below. */
#Fade { grid-column: 1 / -1; position: sticky; bottom: 0; height: var(--size-xl); margin-top: calc(var(--size-xl) * -1); background: linear-gradient(to bottom, transparent, var(--backdrop)); pointer-events: none; opacity: 0; }
@container scroll-state(scrollable: bottom) { #Fade { opacity: 1; } }
#Actions { display: flex; width: stretch; justify-content: flex-end; gap: var(--space-sm); padding-bottom: env(safe-area-inset-bottom); }
#Actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); }
/* Under a finger the buttons are pressed, not keyed: no keycaps. */
@media (pointer: coarse) { #BackKey, #EquipKey { display: none; } }
</style>
