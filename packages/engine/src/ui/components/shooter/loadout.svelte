<meta name="component" content="Loadout">
<meta name="strings" content="./shooter.en.ogt">
<meta name="prop:title" content="string" data-doc="The page's heading; empty takes the kit's (shooter.loadout.title, Loadout).">
<meta name="prop:categories" content="list" data-doc="The slot groups in order, a keyed collection of { id, label }: an empty label takes the kit's string for the id (shooter.loadout.category.<id>), which has primary, secondary, melee and utility.">
<meta name="prop:slots" content="list" data-doc="What can be carried, a keyed collection of { id, category, name, icon, detail, equipped, locked, cost }: category a category's id; icon an icon of the style's atlas; detail a short line under the name (ammunition, damage); equipped it is carried now; locked it cannot be chosen yet (shown with its cost); cost text, empty for none.">
<meta name="prop:selected" content="string" data-doc="The slot id selected now (empty: none).">
<meta name="action:details" content="Equip">
<meta name="action:back" content="Back">
<meta name="parts" content="Loadout.item Loadout.category Loadout.detail Loadout.price">
<meta name="fixture:default" content='{"title":"","selected":"strider","categories":[{"id":"primary","label":""},{"id":"secondary","label":""},{"id":"melee","label":""}],"slots":[{"id":"strider","category":"primary","name":"Strider AR","icon":"sword","detail":"25 / 75","equipped":true,"locked":false,"cost":"2,900"},{"id":"halcyon","category":"primary","name":"Halcyon","icon":"sword","detail":"30 / 90","equipped":false,"locked":false,"cost":"2,900"},{"id":"pike","category":"primary","name":"Pike .50","icon":"sword","detail":"5 / 10","equipped":false,"locked":true,"cost":"4,700"},{"id":"wren","category":"secondary","name":"Wren Pistol","icon":"bolt","detail":"15 / 45","equipped":true,"locked":false,"cost":"500"},{"id":"knife","category":"melee","name":"Field Knife","icon":"sword","detail":"","equipped":true,"locked":false,"cost":""}]}'>
<meta name="fixture:empty" content='{"title":"","selected":"","categories":[{"id":"primary","label":""}],"slots":[]}'>

<section id="Loadout" aria-label={title != '' ? title : t("shooter.loadout.title")}>
  <h2 id="Title">{title != '' ? title : t("shooter.loadout.title")}</h2>
  <div id="Body">
  <div id="Groups">
    {#each categories as c (c.id)}
      <section class="group" aria-label={c.label != '' ? c.label : t("shooter.loadout.category." + c.id)}>
        <h3 class="category" part="Loadout.category">{c.label != '' ? c.label : t("shooter.loadout.category." + c.id)}</h3>
        <div class="items">
          {#each slots as s (s.id)}
            {#if s.category == c.id}
              <button class="item" part="Loadout.item" aria-selected={s.id == selected} aria-label={s.name} class:equipped={s.equipped} class:locked={s.locked} disabled={s.locked} onclick={select} ondetails={equip}><div class="face">
                <og-icon class="icon" icon={s.locked ? 'lock' : s.icon}></og-icon>
                <div class="text">
                  <span class="name">{s.name}</span>
                  <div class="sub" hidden={s.detail == '' && !s.equipped}>
                    <span class="detail" part="Loadout.detail" hidden={s.detail == ''}>{s.detail}</span>
                    <span class="badge" hidden={!s.equipped}>{t("shooter.loadout.equipped")}</span>
                  </div>
                </div>
                {#if s.cost != ''}<div class="cost"><og-icon class="coin" icon="coin"></og-icon><span class="price" part="Loadout.price">{s.cost}</span></div>{/if}
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
  </div>
</section>

<style>
#Loadout { width: stretch; height: stretch; gap: var(--space-lg); }
#Title { font: var(--text-xl2); font-family: var(--font-display); }
#Body { width: stretch; height: stretch; gap: var(--space-lg); }
/* The scrollbar is drawn on the right in either direction, so the room beside it is
   on the right too. */
#Groups { width: stretch; height: stretch; overflow-y: auto; scrollbar-gutter: stable; padding-right: var(--space-lg); container-type: scroll-state; gap: var(--space-xl); display: grid; grid-template-columns: repeat(auto-fill, minmax(calc(var(--size-xxl) * 3), 1fr)); align-content: start; }
.group { gap: var(--space-sm); }
.category { font: var(--text-caption); color: var(--ink-faint); text-transform: uppercase; letter-spacing: 0.08em; }
.items { gap: var(--space-xs); }
.item { width: stretch; height: auto; min-height: var(--size-touch); padding: var(--space-sm) var(--space-md); border-radius: var(--radius-sm); }
.face { display: flex; width: stretch; align-items: center; gap: var(--space-md); }
.icon { flex: none; width: var(--size-sm); height: var(--size-sm); color: var(--ink-muted); }
.text { width: stretch; min-width: 0; gap: var(--space-xxs); align-items: flex-start; }
.name { font: var(--text-body); text-align: start; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: stretch; }
/* The tag drops under the figures when a long translation leaves no room beside them. */
.sub { display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-xxs) var(--space-sm); }
.detail { text-align: start; font: var(--text-caption); font-family: var(--font-mono); white-space: nowrap; }
/* Equipped is a tag on the detail line, so every row's price stands alone at its end
   and the name keeps the row's width. */
.badge { flex: 0 1 auto; max-width: stretch; overflow: hidden; text-overflow: ellipsis; font: var(--text-caption); color: var(--accent); text-transform: uppercase; white-space: nowrap; padding: 0 var(--space-xs); border: 1px solid var(--accent); border-radius: var(--radius-xs); }
.cost { display: flex; flex: none; align-items: center; gap: var(--space-xxs); }
.price { font: var(--text-caption); font-family: var(--font-mono); white-space: nowrap; }
.coin { width: var(--size-xs); height: var(--size-xs); color: var(--gold); }
.locked .name, .locked .icon { color: var(--ink-faint); }
#Empty { font: var(--text-body); color: var(--ink-muted); }
/* The bottom edge fades while there is more to scroll, so a cut row reads as more below. */
#Fade { grid-column: 1 / -1; position: sticky; bottom: 0; height: var(--size-xl); margin-top: calc(var(--size-xl) * -1); background: linear-gradient(to bottom, transparent, var(--backdrop)); pointer-events: none; opacity: 0; }
@container scroll-state(scrollable: bottom) { #Fade { opacity: 1; } }
#Actions { display: flex; width: stretch; justify-content: flex-end; gap: var(--space-sm); padding-bottom: env(safe-area-inset-bottom); }
#Actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); }
/* A landscape phone has height for a few rows only: the title takes less of it,
   and the actions stand beside the list, at its end under the thumb, so the
   list keeps the page's height. */
@media (height < 600px) and (orientation: landscape) {
  #Loadout { gap: var(--space-sm); }
  #Title { font: var(--text-lg); font-family: var(--font-display); margin: 0; }
  .category { margin: 0; }
  #Body { flex-direction: row; gap: var(--space-md); }
  #Groups { gap: var(--space-md); grid-template-columns: minmax(0, 1fr); }
  #Actions { flex-direction: column; width: auto; height: stretch; justify-content: flex-end; }
}
/* Under a finger the buttons are pressed, not keyed: no keycaps. */
@media (pointer: coarse) { #BackKey, #EquipKey { display: none; } }
</style>
