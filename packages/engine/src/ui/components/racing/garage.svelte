<meta name="component" content="RacingGarage">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingGarage.frame RacingGarage.band RacingGarage.title RacingGarage.hero RacingGarage.tile RacingGarage.name RacingGarage.badge RacingGarage.row RacingGarage.picture RacingGarage.cell RacingGarage.headline RacingGarage.caption RacingGarage.well RacingGarage.figure">
<meta name="prop:title" content="string" data-doc="The page's heading; empty takes the kit's (Garage).">
<meta name="prop:vehicles" content="list" data-doc="The cars in the game's order, keyed rows of { id, title, body, image, owned, locked, affordable, price }: title the car's name, body its class line, image a texture (og.load.texture, 0 none); owned the player has it; locked it cannot be had yet; affordable the game allows buying it now (false disables Buy and names it Too costly); price what buying costs or why it is locked, as the game words it (12,500 cr, Reach level 12).">
<meta name="prop:shown" content="string" data-doc="The car in view (its tile selected, its stats shown, the action for it); empty is none.">
<meta name="prop:index" content="number" data-doc="The shown car's place in vehicles, from 0, read out as n of N beside the arrows (the game keeps it with shown).">
<meta name="prop:chosen" content="string" data-doc="The car the player drives now (Driving); empty is none.">
<meta name="prop:stats" content="list" data-doc="The shown car's ratings, keyed rows of { id, label, value, max, text }: label as the game words it (Speed); value over max fills the bar; text the figure shown beside it (8.4).">
<meta name="action:tabPrev" content="Previous car">
<meta name="action:tabNext" content="Next car">
<meta name="action:back" content="Back">
<meta name="fixture:default" content='{"title":"","shown":"gt","index":1,"chosen":"hatch","vehicles":[{"id":"hatch","title":"Pulse Hatch","body":"Class D · Front drive","image":0,"owned":true,"locked":false,"affordable":true,"price":""},{"id":"gt","title":"Corsa GT","body":"Class B · Rear drive","image":0,"owned":true,"locked":false,"affordable":true,"price":""},{"id":"rally","title":"Talon Rally","body":"Class B · All-wheel drive","image":0,"owned":false,"locked":false,"affordable":true,"price":"18,500 cr"},{"id":"proto","title":"Vanta LMP","body":"Class S · Prototype","image":0,"owned":false,"locked":true,"affordable":true,"price":"Reach level 20"}],"stats":[{"id":"speed","label":"Speed","value":7.8,"max":10,"text":"7.8"},{"id":"accel","label":"Acceleration","value":6.9,"max":10,"text":"6.9"},{"id":"handling","label":"Handling","value":8.4,"max":10,"text":"8.4"},{"id":"braking","label":"Braking","value":7.1,"max":10,"text":"7.1"}]}'>
<meta name="fixture:selected" content='{"title":"","shown":"hatch","index":0,"chosen":"hatch","vehicles":[{"id":"hatch","title":"Pulse Hatch","body":"Class D · Front drive","image":0,"owned":true,"locked":false,"affordable":true,"price":""},{"id":"gt","title":"Corsa GT","body":"Class B · Rear drive","image":0,"owned":true,"locked":false,"affordable":true,"price":""}],"stats":[{"id":"speed","label":"Speed","value":5.2,"max":10,"text":"5.2"},{"id":"handling","label":"Handling","value":7.7,"max":10,"text":"7.7"}]}'>
<meta name="fixture:buy" content='{"title":"","shown":"rally","index":1,"chosen":"hatch","vehicles":[{"id":"hatch","title":"Pulse Hatch","body":"Class D · Front drive","image":0,"owned":true,"locked":false,"affordable":true,"price":""},{"id":"rally","title":"Talon Rally","body":"Class B · All-wheel drive","image":0,"owned":false,"locked":false,"affordable":true,"price":"18,500 cr"}],"stats":[{"id":"speed","label":"Speed","value":7.2,"max":10,"text":"7.2"},{"id":"handling","label":"Handling","value":8.9,"max":10,"text":"8.9"}]}'>
<meta name="fixture:short" content='{"title":"","shown":"rally","index":1,"chosen":"hatch","vehicles":[{"id":"hatch","title":"Pulse Hatch","body":"Class D · Front drive","image":0,"owned":true,"locked":false,"affordable":true,"price":""},{"id":"rally","title":"Talon Rally","body":"Class B · All-wheel drive","image":0,"owned":false,"locked":false,"affordable":false,"price":"18,500 cr"}],"stats":[{"id":"speed","label":"Speed","value":7.2,"max":10,"text":"7.2"},{"id":"handling","label":"Handling","value":8.9,"max":10,"text":"8.9"}]}'>
<meta name="fixture:locked" content='{"title":"","shown":"proto","index":1,"chosen":"hatch","vehicles":[{"id":"hatch","title":"Pulse Hatch","body":"Class D · Front drive","image":0,"owned":true,"locked":false,"affordable":true,"price":""},{"id":"proto","title":"Vanta LMP","body":"Class S · Prototype","image":0,"owned":false,"locked":true,"affordable":true,"price":"Reach level 20"}],"stats":[{"id":"speed","label":"Speed","value":9.8,"max":10,"text":"9.8"},{"id":"handling","label":"Handling","value":9.1,"max":10,"text":"9.1"}]}'>
<meta name="fixture:empty" content='{"title":"","shown":"","index":0,"chosen":"","vehicles":[],"stats":[]}'>
<meta name="fixture:overflow" content='{"title":"Garage of the Grand Touring Championship","shown":"v7","index":6,"chosen":"v1","vehicles":[{"id":"v1","title":"Pulse Hatch","body":"Class D","image":0,"owned":true,"locked":false,"affordable":true,"price":""},{"id":"v2","title":"Corsa GT","body":"Class B","image":0,"owned":true,"locked":false,"affordable":true,"price":""},{"id":"v3","title":"Talon Rally","body":"Class B","image":0,"owned":false,"locked":false,"affordable":true,"price":"18,500 cr"},{"id":"v4","title":"Vanta LMP","body":"Class S","image":0,"owned":false,"locked":true,"affordable":true,"price":"Reach level 20"},{"id":"v5","title":"Kestrel Roadster","body":"Class C","image":0,"owned":true,"locked":false,"affordable":true,"price":""},{"id":"v6","title":"Basalt Truck","body":"Class D","image":0,"owned":false,"locked":false,"affordable":true,"price":"9,900 cr"},{"id":"v7","title":"Meridian Hyperion Twin-Turbo Competizione Evoluzione","body":"Class S · Mid-engine · Carbon monocoque · Limited series","image":0,"owned":true,"locked":false,"affordable":true,"price":""},{"id":"v8","title":"Nimbus EV","body":"Class A","image":0,"owned":false,"locked":true,"affordable":true,"price":"Win the Coastal Cup"},{"id":"v9","title":"Ridge Buggy","body":"Class C","image":0,"owned":false,"locked":false,"affordable":true,"price":"6,200 cr"},{"id":"v10","title":"Solace Coupe","body":"Class A","image":0,"owned":true,"locked":false,"affordable":true,"price":""}],"stats":[{"id":"speed","label":"Top speed","value":9.9,"max":10,"text":"9.9"},{"id":"accel","label":"Acceleration","value":9.6,"max":10,"text":"9.6"},{"id":"handling","label":"Handling","value":7.4,"max":10,"text":"7.4"},{"id":"braking","label":"Braking","value":8.8,"max":10,"text":"8.8"},{"id":"offroad","label":"Off-road","value":1.2,"max":10,"text":"1.2"}]}'>

<!--
  Choosing a car: the shown car large (its picture, name and class) with arrows to step the cars (prev, next: the arrows, Q and E, the bumpers), a strip of tiles to
  jump to one (choose), the shown car's ratings as bars, and one action for
  it: Drive (select) when owned, Buy with its price (buy) when it can be
  had (disabled and named Too costly when the game says it is not
  affordable), a disabled Locked with the reason when it cannot, Driving when it is
  already the player's car. The game owns prices, unlocking and every
  rating; the garage reports the car's key and Back (back).
-->
<section id="RacingGarage" part="RacingGarage.frame" data-layout="shell" data-gap="loose" aria-label={title != '' ? title : t("racing.garage.title")}>
  {#each vehicles as v (v.id)}{#if v.id == shown && v.image}<img class="backdrop" src={v.image} alt="" data-anchor="fill">{/if}{/each}
  <div class="shade" data-anchor="fill"></div>
  <header part="RacingGarage.band" data-layout="row"><h2 part="RacingGarage.title" data-fill>{title != '' ? title : t("racing.garage.title")}</h2></header>
  {#if vehicles.length}
    <div class="body" data-layout="switcher" data-gap="wide" data-fill>
      <div class="show" data-layout="stack" data-fill ontabPrev={prev} ontabNext={next}>
        {#each vehicles as v (v.id)}
          {#if v.id == shown}
            <figure part="RacingGarage.hero RacingGarage.picture" class="hero" data-layout="stack" data-gap="hair" aria-label={v.title} data-owned={v.owned} data-locked={v.locked} data-chosen={v.id == chosen}>
              {#if !v.image}<div class="well" part="RacingGarage.well"></div>{/if}
              <span class="hero-name" part="RacingGarage.headline">{v.title}</span>
              <span class="hero-body" part="RacingGarage.caption">{v.body}</span>
            </figure>
          {/if}
        {/each}
        <div class="steps" data-layout="row" data-gap="wide">
          <button id="Prev" variant="ghost" aria-label={t("racing.garage.prev")} onclick={prev}><og-icon icon="back"></og-icon></button>
          <span class="count" part="RacingGarage.figure">{t("racing.garage.count", { n: index + 1, total: vehicles.length })}</span>
          <button id="Next" variant="ghost" aria-label={t("racing.garage.next")} onclick={next}><og-icon class="next-icon" icon="back"></og-icon></button>
        </div>
      </div>
      <div class="side" data-layout="stack" data-gap="wide" data-fill>
        <div class="stats" data-layout="stack" role="list" aria-label={t("racing.garage.stats")}>
          {#each stats as s (s.id)}
            <div part="RacingGarage.row" data-layout="row" role="listitem"><StatBar value={s.value} max={s.max} kind="stamina" segment={0} shield={0} shieldMax={0} heal={0} recoverable={0} label={s.label} icon="" showValue={true} valueInside={false} valueText={s.text} low={0} critical={0} bare={true} name={s.label} /></div>
          {/each}
        </div>
        <div class="actions" data-layout="cluster">
          <button id="Back" variant="ghost" aria-label={t("racing.garage.back")} onclick={back} onback={back}><kbd class="key" data-action="back"></kbd><span>{t("racing.garage.back")}</span></button>
          {#each vehicles as v (v.id)}
            {#if v.id == shown}
              {#if v.locked}
                <button id="Locked" class="go" variant="primary" disabled={true} aria-description={v.price}><span>{t("racing.garage.locked")}</span>{#if v.price}<span class="reason">{v.price}</span>{/if}</button>
              {:else}
                {#if v.owned}
                  <button id="Select" class="go" variant="primary" disabled={v.id == chosen} onclick={select}><span>{v.id == chosen ? t("racing.garage.driving") : t("racing.garage.select")}</span></button>
                {:else}
                  <button id="Buy" class="go" variant="primary" disabled={!v.affordable} onclick={buy}><span>{t("racing.garage.buy", { price: v.price })}</span>{#if !v.affordable}<span class="reason">{t("racing.garage.short")}</span>{/if}</button>
                {/if}
              {/if}
            {/if}
          {/each}
        </div>
      </div>
    </div>
    <div class="tiles-scroll">
      <div class="tiles" data-layout="row" role="listbox" aria-label={t("racing.garage.vehicles")}>
        {#each vehicles as v (v.id)}
          <button part="RacingGarage.tile RacingGarage.cell" role="option" aria-selected={v.id == shown} aria-label={v.title} data-owned={v.owned} data-locked={v.locked} data-chosen={v.id == chosen} onclick={choose}><div class="face" data-layout="stack" data-gap="hair">
            <span part="RacingGarage.name" class="name">{v.title}</span>
            <span part="RacingGarage.badge" class="tag" data-tag={v.id == chosen ? 'driving' : (v.locked ? 'locked' : (v.owned ? 'owned' : 'price'))}>{v.id == chosen ? t("racing.garage.driving") : (v.locked ? t("racing.garage.locked") : (v.owned ? t("racing.garage.owned") : v.price))}</span>
          </div></button>
        {/each}
      </div>
    </div>
  {:else}
    <div class="empty" data-layout="cover" data-gap="loose" data-fill role="status">
      <p class="empty-text" part="RacingGarage.caption">{t("racing.garage.empty")}</p>
      <button id="EmptyBack" variant="ghost" aria-label={t("racing.garage.back")} onclick={back} onback={back}><kbd class="key" data-action="back"></kbd><span>{t("racing.garage.back")}</span></button>
    </div>
  {/if}
</section>

<style>
/* The garage: the shown car's picture fills the page under a shade that
   holds the words over it; its class in the accent over its name in large
   slanted capitals, the arrows and the count under them; the ratings and
   the action on a smoked cut card at the end; the cars along the foot as
   cut tiles, the shown one ringed in the accent. */
:host { color: var(--hud-ink); }
[part="RacingGarage.frame"] { position: relative; box-sizing: border-box; padding: var(--space-xl); border: 0; border-radius: 0; overflow: hidden; color: var(--hud-ink); background: var(--party-ground); box-shadow: none; }
.backdrop { inline-size: 100%; block-size: 100%; object-fit: cover; }
.shade { background: var(--party-hero-shade); pointer-events: none; }
[part="RacingGarage.band"], .body, .tiles-scroll, .empty { position: relative; }
[part="RacingGarage.band"] { width: stretch; flex: none; min-height: 0; padding: 0; border: 0; border-radius: 0; background: none; box-shadow: none; }
[part="RacingGarage.title"] { margin: 0; font-size: var(--hud-label-size); line-height: 1.3; font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); color: var(--hud-ink); text-shadow: var(--front-shadow); overflow-wrap: break-word; }
.body { width: stretch; min-height: 0; overflow-y: auto; align-items: safe flex-end; justify-content: space-between; -og-scroll-fade: var(--space-xl); --layout-switch-at: calc(var(--party-garage-side) * 2); }
.show { align-items: flex-start; min-width: 0; max-width: stretch; }
[part="RacingGarage.hero RacingGarage.picture"] { width: stretch; margin: 0; padding: 0; border: 0; border-radius: 0; background: none; box-shadow: none; --layout-gap-hair: var(--space-xs); }
.well { order: -2; width: stretch; aspect-ratio: 3.2; background: var(--party-open-row); box-shadow: var(--party-seat-rim); border-radius: var(--party-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.hero-body { order: -1; width: stretch; text-align: start; font-size: var(--hud-label-size); line-height: 1.3; font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); color: var(--hud-accent); text-shadow: var(--front-shadow); overflow-wrap: break-word; }
.hero-name { width: stretch; text-align: start; font-size: var(--party-size-car); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-style: var(--hud-figure-slant); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: var(--front-shadow); overflow-wrap: break-word; line-clamp: 3; overflow: hidden; }
.steps button { width: var(--size-touch); height: var(--size-touch); padding: 0; border: 0; color: var(--hud-ink); background: var(--party-button-fill); box-shadow: var(--party-button-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; --button-radius: var(--hud-plate-cut) 0; --button-corner-shape: var(--cut-shape); }
.steps button:hover { background: var(--party-button-hover-fill); }
.steps button:focus-visible { box-shadow: var(--party-focus-rim); }
.next-icon { rotate: 180deg; }
#Prev:dir(rtl) og-icon { rotate: 180deg; }
#Next:dir(rtl) .next-icon { rotate: 0deg; }
.count { min-width: var(--size-xl); text-align: center; white-space: nowrap; font-size: var(--hud-size-readout); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); text-shadow: var(--front-shadow); }
/* The card takes the ratings' share beside the car, or the whole width
   once the garage is too narrow for both at the player's text size. */
.side { flex-grow: 0.6; box-sizing: border-box; max-width: stretch; padding: var(--space-md); background: var(--party-card-fill); box-shadow: var(--party-card-rim); --layout-gap-wide: var(--space-md); border-radius: var(--party-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
[part="RacingGarage.row"] { border-radius: 0; background: none; }
.actions { justify-content: flex-end; }
.actions button, #EmptyBack { min-height: var(--size-touch); gap: var(--space-xs); padding: 0 var(--space-lg); border: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--party-button-fill); box-shadow: var(--party-button-rim); text-shadow: none; border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; --button-radius: var(--hud-plate-cut) 0; --button-corner-shape: var(--cut-shape); }
.actions button { flex: 1 1 auto; flex-wrap: wrap; min-width: calc(var(--size-xxl) * 1.25); padding-block: var(--space-xxs); }
.actions button:hover, #EmptyBack:hover { background: var(--party-button-hover-fill); }
.actions button:focus-visible, #EmptyBack:focus-visible { box-shadow: var(--party-focus-rim); }
.actions button:disabled { opacity: 0.45; }
.actions button > span, #EmptyBack > span { color: inherit; font-weight: inherit; }
.reason { overflow-wrap: break-word; }
/* The strip scrolls inside the garage's width and never widens it. */
.tiles-scroll { overflow-x: auto; min-width: 0; flex: none; contain: inline-size; }
.tiles { padding: var(--space-xxs); align-items: stretch; }
[part="RacingGarage.tile RacingGarage.cell"] { flex: none; width: var(--party-garage-tile); min-height: var(--size-xl); padding: var(--space-xs) var(--space-sm); border: 0; color: var(--hud-ink); background: var(--party-card-fill); box-shadow: var(--party-tile-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
[part="RacingGarage.tile RacingGarage.cell"]:hover { box-shadow: var(--party-tile-hover-rim); }
[part="RacingGarage.tile RacingGarage.cell"]:focus-visible { box-shadow: var(--party-focus-rim); }
[part="RacingGarage.tile RacingGarage.cell"][aria-selected=true] { background: var(--party-card-on-fill); box-shadow: var(--party-on-rim); }
[part="RacingGarage.tile RacingGarage.cell"][data-locked=true] { color: var(--hud-ink-dim); }
[part="RacingGarage.tile RacingGarage.cell"] .face { width: stretch; align-items: flex-start; }
.name { width: stretch; text-align: start; font-size: var(--hud-size-xs); line-height: 1.15; font-weight: var(--hud-figure-weight); color: inherit; overflow-wrap: break-word; }
.tag { max-width: stretch; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); overflow-wrap: break-word; border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.tag[data-tag=driving] { color: var(--party-go-ink); background: var(--hud-accent); box-shadow: none; }
.tag[data-tag=price] { color: var(--hud-gold); }
.tag[data-tag=locked] { color: var(--hud-ink-dim); }
.empty-text { margin: 0; font-size: var(--hud-size-sm); color: var(--hud-ink-dim); }
.key { color: var(--hud-key-ink); background: var(--hud-key-fill); box-shadow: var(--hud-key-shadow); border-radius: var(--hud-key-radius); }
@media (pointer: coarse) { .key { display: none; } }
/* A phone: the picture closes on the ground under it, the name smaller.
   The frame's own gap follows the viewport: a container cannot query
   itself. */
@media (width < 600px) {
  [part="RacingGarage.frame"] { padding: var(--space-sm); --layout-gap-loose: var(--layout-gap); }
  .backdrop, .shade { bottom: auto; block-size: 38%; }
  .shade { background: var(--party-hero-shade-compact); }
  [part="RacingGarage.tile RacingGarage.cell"] { width: var(--party-garage-tile-compact); }
}
/* A narrow panel stacks the shown car over the ratings and the action, the
   tiles along the foot under the thumb. */
@container (width < 600px) {
  .body { flex-direction: column; flex-wrap: nowrap; align-items: stretch; justify-content: safe flex-end; --layout-gap-wide: var(--layout-gap); }
  .show { width: stretch; flex: none; }
  .side { width: stretch; flex: none; padding: var(--space-sm); --layout-gap-wide: var(--layout-gap); }
  .actions { width: stretch; }
}
</style>
