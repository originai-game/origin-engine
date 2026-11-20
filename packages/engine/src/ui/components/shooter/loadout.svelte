<meta name="component" content="Loadout">
<meta name="strings" content="./shooter.en.ogt">
<meta name="prop:title" content="string" data-doc="The page's heading; empty takes the kit's (shooter.loadout.title, Loadout).">
<meta name="prop:categories" content="list" data-doc="The slot groups in order, a keyed collection of { id, label }: an empty label takes the kit's string for the id (shooter.loadout.category.<id>), which has primary, secondary, melee and utility.">
<meta name="prop:slots" content="list" data-doc="What can be carried, a keyed collection of { id, category, name, icon, detail, equipped, locked, cost }: category a category's id; icon an icon of the style's atlas; detail a short line under the name (ammunition, damage); equipped it is carried now; locked it cannot be chosen yet (shown with its cost); cost text, empty for none.">
<meta name="prop:selected" content="string" data-doc="The slot id selected now (empty: none).">
<meta name="action:details" content="Equip">
<meta name="action:back" content="Back">
<meta name="fixture:default" content='{"title":"","selected":"strider","categories":[{"id":"primary","label":""},{"id":"secondary","label":""},{"id":"melee","label":""}],"slots":[{"id":"strider","category":"primary","name":"Strider AR","icon":"sword","detail":"25 / 75","equipped":true,"locked":false,"cost":"2,900"},{"id":"halcyon","category":"primary","name":"Halcyon","icon":"sword","detail":"30 / 90","equipped":false,"locked":false,"cost":"2,900"},{"id":"pike","category":"primary","name":"Pike .50","icon":"sword","detail":"5 / 10","equipped":false,"locked":true,"cost":"4,700"},{"id":"wren","category":"secondary","name":"Wren Pistol","icon":"bolt","detail":"15 / 45","equipped":true,"locked":false,"cost":"500"},{"id":"knife","category":"melee","name":"Field Knife","icon":"sword","detail":"","equipped":true,"locked":false,"cost":""}]}'>
<meta name="fixture:empty" content='{"title":"","selected":"","categories":[{"id":"primary","label":""}],"slots":[]}'>

<section id="Loadout" layout="stack" fill gap="wide" aria-label={title != '' ? title : t("shooter.loadout.title")}>
  <h2 id="Title" part="title">{title != '' ? title : t("shooter.loadout.title")}</h2>
  <div class="grid" id="Body" gap="wide" fill>
  <div class="grid" id="Groups" gap="wide" fill>
    {#each categories as c (c.id)}
      <section class="group" layout="stack" aria-label={c.label ? c.label : t("shooter.loadout.category." + c.id)}>
        <h3 class="category" part="label">{c.label ? c.label : t("shooter.loadout.category." + c.id)}</h3>
        <div class="items" layout="stack" gap="tight">
          {#each slots as s (s.id)}
            {#if s.category == c.id}
              <button class="item" part="cell" aria-selected={s.id == selected} aria-label={s.name} class:equipped={s.equipped} class:locked={s.locked} disabled={s.locked} onclick={select} ondetails={equip} state:selected={s.id == selected}><div class="face" layout="row" gap="loose">
                <og-icon class="icon" icon={s.locked ? 'lock' : s.icon}></og-icon>
                <div class="text" layout="stack" gap="hair" fill>
                  <span class="name" part="headline">{s.name}</span>
                  {#if s.detail || s.equipped}<div class="sub" layout="wrap" gap="hair">
                    {#if s.detail}<span class="detail" part="figure">{s.detail}</span>{/if}
                    {#if s.equipped}<span class="badge" part="badge" tone="success">{t("shooter.loadout.equipped")}</span>{/if}
                  </div>{/if}
                </div>
                {#if s.cost}<div class="cost" layout="row" gap="hair"><og-icon class="coin" icon="coin"></og-icon><span class="price" part="figure">{s.cost}</span></div>{/if}
              </div></button>
            {/if}
          {/each}
        </div>
      </section>
    {/each}
    {#if slots.length == 0}
      <p id="Empty" part="caption">{t("shooter.loadout.empty")}</p>
    {/if}
    <div id="Fade" part="fade"></div>
  </div>
  <footer id="Actions" layout="wrap">
    <button id="Back" variant="ghost" aria-label={t("shooter.loadout.back")} onclick={back} onback={back}><kbd id="BackKey" data-action="back"></kbd><span id="BackLabel">{t("shooter.loadout.back")}</span></button>
    {#each slots as s (s.id)}
      {#if s.id == selected}
        <button id="Equip" class="go" variant="primary" aria-label={t("shooter.loadout.equip")} disabled={s.locked || s.equipped} onclick={equip}><kbd id="EquipKey" data-action="details"></kbd><span id="EquipLabel">{t("shooter.loadout.equip")}</span></button>
      {/if}
    {/each}
  </footer>
  </div>
</section>

<style>
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(var(--layout-tile), 100%), 1fr)); }
/* The loadout over the scene: the title in large condensed capitals, a
   category's name over a rule, each item a cut plate of smoked glass (the
   chosen one ringed in the accent and washed in it), Equipped on a green
   tag, the price in gold; names wrap on whole words. */
#Loadout { color: var(--hud-ink); }
#Title { margin: 0; font-size: var(--party-size-hero); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: var(--front-shadow); overflow-wrap: break-word; }
#Body { width: stretch; grid-template-columns: minmax(0, 1fr); grid-template-rows: minmax(0, 1fr) auto; }
/* The scrollbar is drawn on the right in either direction, so the room beside it is
   on the right too. */
#Groups { width: stretch; overflow-y: auto; scrollbar-gutter: stable; padding-right: var(--space-lg); --layout-tile: calc(var(--size-xxl) * 3); align-content: start; }
.group { min-width: 0; }
.category { margin: 0; padding-block-end: var(--space-xs); font-size: var(--hud-label-size); line-height: 1.3; font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); color: var(--hud-ink); box-shadow: var(--party-rule-strong); text-shadow: var(--front-shadow); overflow-wrap: break-word; }
.item { width: stretch; height: auto; min-height: var(--size-touch); padding: var(--space-sm) var(--space-md); min-width: 0; border: 0; color: var(--hud-ink); background: var(--party-card-fill); box-shadow: var(--party-tile-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.item:hover { box-shadow: var(--party-tile-hover-rim); }
.item:focus-visible { box-shadow: var(--party-focus-rim); }
.item:state(selected) { background: var(--party-card-on-fill); box-shadow: var(--party-on-rim); }
.item:disabled { color: var(--hud-ink-dim); }
.face { width: stretch; }
.icon { flex: none; width: var(--size-sm); height: var(--size-sm); color: inherit; }
.text { min-width: 0; align-items: flex-start; }
.name { max-width: stretch; text-align: start; font-size: var(--hud-size-name); line-height: 1.15; font-weight: var(--hud-figure-weight); color: inherit; overflow-wrap: break-word; }
.detail { text-align: start; white-space: nowrap; font-size: var(--hud-size-xs); font-weight: var(--hud-label-weight); color: var(--hud-ink-dim); text-shadow: none; }
/* Equipped is a tag on the detail line, so every row's price stands alone at its end
   and the name keeps the row's width. */
.badge { flex: 0 1 auto; max-width: stretch; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--party-ready-ink); background: var(--party-ready-fill); box-shadow: none; overflow-wrap: break-word; border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.cost { flex: none; }
.price { white-space: nowrap; font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-gold); text-shadow: none; }
.item:disabled .price { color: var(--hud-ink-dim); }
.coin { width: var(--size-xs); height: var(--size-xs); color: var(--hud-gold); }
.item:disabled .coin { color: var(--hud-ink-dim); }
#Empty { margin: 0; font-size: var(--hud-size-sm); color: var(--hud-ink-dim); }
/* The bottom edge fades while there is more to scroll, so a cut row reads as more below. */
#Fade { grid-column: 1 / -1; position: sticky; bottom: 0; margin-top: calc(var(--fade-size) * -1); opacity: 0; }
#Groups:state(more) #Fade { opacity: 1; }
#Actions { justify-content: flex-end; padding-bottom: env(safe-area-inset-bottom); }
#Actions button { min-height: var(--size-touch); gap: var(--space-xs); padding: 0 var(--space-lg); border: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--party-button-fill); box-shadow: var(--party-button-rim); text-shadow: none; border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; --button-radius: var(--hud-plate-cut) 0; --button-corner-shape: var(--cut-shape); min-width: calc(var(--size-xxl) * 1.25); }
#Actions button:hover { background: var(--party-button-hover-fill); }
#Actions button:focus-visible { box-shadow: var(--party-focus-rim); }
#Actions button:disabled { opacity: 0.45; }
#Actions button span { color: inherit; font-weight: inherit; }
#BackKey, #EquipKey { color: var(--hud-key-ink); background: var(--hud-key-fill); box-shadow: var(--hud-key-shadow); border-radius: var(--hud-key-radius); }
#Loadout:size(compact) { --layout-gap-wide: var(--layout-gap-loose); }
#Title:size(compact) { font-size: var(--party-size-hero-compact); }
#Groups:size(compact) { padding-right: var(--space-xs); }
#Actions button:size(compact) { flex: 1 1 auto; }
/* A landscape phone has height for a few rows only: the title takes less of it,
   and the actions stand beside the list, at its end under the thumb, so the
   list keeps the page's height. */
#Loadout:size(short) { --layout-gap-wide: var(--layout-gap); }
#Title:size(short) { font-size: var(--party-size-hero-compact); }
#Body:size(short) { grid-template-columns: minmax(0, 1fr) auto; grid-template-rows: minmax(0, 1fr); }
#Groups:size(short) { grid-template-columns: minmax(0, 1fr); }
#Actions:size(short) { width: calc(var(--size-xxl) * 1.25); height: stretch; align-content: end; }
/* Under a finger the buttons are pressed, not keyed: no keycaps. */
#BackKey:touch, #EquipKey:touch { display: none; }
</style>
