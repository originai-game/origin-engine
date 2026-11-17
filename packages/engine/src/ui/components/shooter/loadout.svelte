<meta name="component" content="Loadout">
<meta name="strings" content="./shooter.en.ogt">
<meta name="prop:title" content="string" data-doc="The page's heading; empty takes the kit's (shooter.loadout.title, Loadout).">
<meta name="prop:categories" content="list" data-doc="The slot groups in order, a keyed collection of { id, label }: an empty label takes the kit's string for the id (shooter.loadout.category.<id>), which has primary, secondary, melee and utility.">
<meta name="prop:slots" content="list" data-doc="What can be carried, a keyed collection of { id, category, name, icon, detail, equipped, locked, cost }: category a category's id; icon an icon of the style's atlas; detail a short line under the name (ammunition, damage); equipped it is carried now; locked it cannot be chosen yet (shown with its cost); cost text, empty for none.">
<meta name="prop:selected" content="string" data-doc="The slot id selected now (empty: none).">
<meta name="action:details" content="Equip">
<meta name="action:back" content="Back">
<meta name="parts" content="Loadout.item Loadout.category Loadout.detail Loadout.price Loadout.label Loadout.cell Loadout.figure Loadout.title Loadout.headline Loadout.caption Loadout.badge Loadout.fade">
<meta name="fixture:default" content='{"title":"","selected":"strider","categories":[{"id":"primary","label":""},{"id":"secondary","label":""},{"id":"melee","label":""}],"slots":[{"id":"strider","category":"primary","name":"Strider AR","icon":"sword","detail":"25 / 75","equipped":true,"locked":false,"cost":"2,900"},{"id":"halcyon","category":"primary","name":"Halcyon","icon":"sword","detail":"30 / 90","equipped":false,"locked":false,"cost":"2,900"},{"id":"pike","category":"primary","name":"Pike .50","icon":"sword","detail":"5 / 10","equipped":false,"locked":true,"cost":"4,700"},{"id":"wren","category":"secondary","name":"Wren Pistol","icon":"bolt","detail":"15 / 45","equipped":true,"locked":false,"cost":"500"},{"id":"knife","category":"melee","name":"Field Knife","icon":"sword","detail":"","equipped":true,"locked":false,"cost":""}]}'>
<meta name="fixture:empty" content='{"title":"","selected":"","categories":[{"id":"primary","label":""}],"slots":[]}'>

<section id="Loadout" data-layout="shell" data-gap="wide" aria-label={title != '' ? title : t("shooter.loadout.title")}>
  <h2 id="Title" part="Loadout.title">{title != '' ? title : t("shooter.loadout.title")}</h2>
  <div id="Body" data-layout="grid" data-gap="wide" data-fill>
  <div id="Groups" data-layout="grid" data-gap="wide" data-fill>
    {#each categories as c (c.id)}
      <section class="group" data-layout="stack" aria-label={c.label != '' ? c.label : t("shooter.loadout.category." + c.id)}>
        <h3 class="category" part="Loadout.category Loadout.label">{c.label != '' ? c.label : t("shooter.loadout.category." + c.id)}</h3>
        <div class="items" data-layout="stack" data-gap="tight">
          {#each slots as s (s.id)}
            {#if s.category == c.id}
              <button class="item" part="Loadout.item Loadout.cell" aria-selected={s.id == selected} aria-label={s.name} class:equipped={s.equipped} class:locked={s.locked} disabled={s.locked} onclick={select} ondetails={equip}><div class="face" data-layout="row" data-gap="loose">
                <og-icon class="icon" icon={s.locked ? 'lock' : s.icon}></og-icon>
                <div class="text" data-layout="stack" data-gap="hair" data-fill>
                  <span class="name" part="Loadout.headline">{s.name}</span>
                  {#if s.detail != '' || s.equipped}<div class="sub" data-layout="cluster" data-gap="hair">
                    {#if s.detail != ''}<span class="detail" part="Loadout.detail Loadout.figure">{s.detail}</span>{/if}
                    {#if s.equipped}<span class="badge" part="Loadout.badge" data-tone="success">{t("shooter.loadout.equipped")}</span>{/if}
                  </div>{/if}
                </div>
                {#if s.cost != ''}<div class="cost" data-layout="row" data-gap="hair"><og-icon class="coin" icon="coin"></og-icon><span class="price" part="Loadout.price Loadout.figure">{s.cost}</span></div>{/if}
              </div></button>
            {/if}
          {/each}
        </div>
      </section>
    {/each}
    {#if slots.length == 0}
      <p id="Empty" part="Loadout.caption">{t("shooter.loadout.empty")}</p>
    {/if}
    <div id="Fade" part="Loadout.fade"></div>
  </div>
  <footer id="Actions" data-layout="cluster">
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
#Body { width: stretch; grid-template-columns: minmax(0, 1fr); grid-template-rows: minmax(0, 1fr) auto; }
/* The scrollbar is drawn on the right in either direction, so the room beside it is
   on the right too. */
#Groups { width: stretch; overflow-y: auto; scrollbar-gutter: stable; padding-right: var(--space-lg); container-type: scroll-state; --layout-tile: calc(var(--size-xxl) * 3); align-content: start; }
.item { width: stretch; height: auto; min-height: var(--size-touch); padding: var(--space-sm) var(--space-md); min-width: 0; }
.face { width: stretch; }
.icon { flex: none; width: var(--size-sm); height: var(--size-sm); }
.text { align-items: flex-start; }
.name { text-align: start; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: stretch; }
/* The tag drops under the figures when a long translation leaves no room beside them. */
.detail { text-align: start; white-space: nowrap; --figure-size: var(--font-step-sm); }
/* Equipped is a tag on the detail line, so every row's price stands alone at its end
   and the name keeps the row's width. */
.badge { flex: 0 1 auto; max-width: stretch; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; padding: 0 var(--space-xs); }
.cost { flex: none; }
.price { --figure-size: var(--font-step-sm); white-space: nowrap; }
.coin { width: var(--size-xs); height: var(--size-xs); }
/* The bottom edge fades while there is more to scroll, so a cut row reads as more below. */
#Fade { grid-column: 1 / -1; position: sticky; bottom: 0; margin-top: calc(var(--fade-size) * -1); opacity: 0; }
@container scroll-state(scrollable: bottom) { #Fade { opacity: 1; } }
#Actions { justify-content: flex-end; padding-bottom: env(safe-area-inset-bottom); }
#Actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); }
/* A landscape phone has height for a few rows only: the title takes less of it,
   and the actions stand beside the list, at its end under the thumb, so the
   list keeps the page's height. */
@media (height < 600px) and (orientation: landscape) {
  :host { --layout-gap-wide: var(--layout-gap); --title-size: var(--title-lead-size); }
  .category { margin: 0; }
  #Body { grid-template-columns: minmax(0, 1fr) auto; grid-template-rows: minmax(0, 1fr); }
  #Groups { grid-template-columns: minmax(0, 1fr); }
  #Actions { width: calc(var(--size-xxl) * 1.25); height: stretch; align-content: end; }
}
/* Under a finger the buttons are pressed, not keyed: no keycaps. */
@media (pointer: coarse) { #BackKey, #EquipKey { display: none; } }
</style>
