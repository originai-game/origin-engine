<meta name="component" content="RacingGarage">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingGarage.frame RacingGarage.band RacingGarage.title RacingGarage.hero RacingGarage.tile RacingGarage.name RacingGarage.badge RacingGarage.row">
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
<section id="RacingGarage" part="RacingGarage.frame" aria-label={title != '' ? title : t("racing.garage.title")}>
  <header part="RacingGarage.band"><h2 part="RacingGarage.title">{title != '' ? title : t("racing.garage.title")}</h2></header>
  {#if vehicles.length}
    <div class="body">
      <div class="show" ontabPrev={prev} ontabNext={next}>
        {#each vehicles as v (v.id)}
          {#if v.id == shown}
            <figure part="RacingGarage.hero" class="hero" aria-label={v.title} data-owned={v.owned} data-locked={v.locked} data-chosen={v.id == chosen}>
              <img class="art" src={v.image} alt="" hidden={!v.image}>
              <div class="well" hidden={!!v.image}></div>
              <span class="hero-name">{v.title}</span>
              <span class="hero-body">{v.body}</span>
            </figure>
          {/if}
        {/each}
        <div class="steps">
          <button id="Prev" variant="ghost" aria-label={t("racing.garage.prev")} onclick={prev}><og-icon icon="back"></og-icon></button>
          <span class="count">{t("racing.garage.count", { n: index + 1, total: vehicles.length })}</span>
          <button id="Next" variant="ghost" aria-label={t("racing.garage.next")} onclick={next}><og-icon class="next-icon" icon="back"></og-icon></button>
        </div>
      </div>
      <div class="side">
        <div class="stats" role="list" aria-label={t("racing.garage.stats")}>
          {#each stats as s (s.id)}
            <div part="RacingGarage.row" role="listitem"><StatBar value={s.value} max={s.max} kind="stamina" segment={0} shield={0} shieldMax={0} heal={0} recoverable={0} label={s.label} icon="" showValue={true} valueInside={false} valueText={s.text} low={0} critical={0} bare={true} name={s.label} /></div>
          {/each}
        </div>
        <div class="actions">
          <button id="Back" variant="ghost" onclick={back} onback={back}><kbd class="key" data-action="back"></kbd><span>{t("racing.garage.back")}</span></button>
          {#each vehicles as v (v.id)}
            {#if v.id == shown}
              {#if v.locked}
                <button id="Locked" variant="primary" disabled={true} aria-description={v.price}><span>{t("racing.garage.locked")}</span><span class="reason" hidden={v.price == ''}>{v.price}</span></button>
              {:else}
                {#if v.owned}
                  <button id="Select" variant="primary" disabled={v.id == chosen} onclick={select}><span>{v.id == chosen ? t("racing.garage.driving") : t("racing.garage.select")}</span></button>
                {:else}
                  <button id="Buy" variant="primary" disabled={!v.affordable} onclick={buy}><span>{t("racing.garage.buy", { price: v.price })}</span><span class="reason" hidden={v.affordable}>{t("racing.garage.short")}</span></button>
                {/if}
              {/if}
            {/if}
          {/each}
        </div>
      </div>
    </div>
    <scroll class="tiles-scroll">
      <div class="tiles" role="listbox" aria-label={t("racing.garage.vehicles")}>
        {#each vehicles as v (v.id)}
          <button part="RacingGarage.tile" role="option" aria-selected={v.id == shown} aria-label={v.title} data-owned={v.owned} data-locked={v.locked} data-chosen={v.id == chosen} onclick={choose}><div class="face">
            <span part="RacingGarage.name" class="name">{v.title}</span>
            <span part="RacingGarage.badge" class="tag" data-tag={v.id == chosen ? 'driving' : (v.locked ? 'locked' : (v.owned ? 'owned' : 'price'))}>{v.id == chosen ? t("racing.garage.driving") : (v.locked ? t("racing.garage.locked") : (v.owned ? t("racing.garage.owned") : v.price))}</span>
          </div></button>
        {/each}
      </div>
    </scroll>
  {:else}
    <div class="empty" role="status">
      <p class="empty-text">{t("racing.garage.empty")}</p>
      <button id="EmptyBack" variant="ghost" onclick={back} onback={back}><kbd class="key" data-action="back"></kbd><span>{t("racing.garage.back")}</span></button>
    </div>
  {/if}
</section>

<style>
[part="RacingGarage.frame"][hidden], [part="RacingGarage.frame"] [hidden] { display: none; }
[part="RacingGarage.frame"] { display: flex; flex-direction: column; width: stretch; min-width: 0; height: stretch; gap: var(--space-md); container-type: inline-size; }
[part="RacingGarage.band"] { display: flex; width: stretch; align-items: center; flex: none; }
[part="RacingGarage.title"] { margin: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.body { display: flex; width: stretch; flex: 1; min-height: 0; gap: var(--space-xl); align-items: flex-start; justify-content: center; }
.show { flex: none; align-items: center; gap: var(--space-sm); width: calc(var(--size-xxl) * 3.6); max-width: stretch; }
[part="RacingGarage.hero"] { display: flex; flex-direction: column; width: stretch; margin: 0; gap: var(--space-xxs); padding: var(--space-sm); }
.art, .well { width: stretch; aspect-ratio: 1.8; }
.art { object-fit: cover; }
.hero-name, .hero-body { width: stretch; text-align: start; }
.hero-name { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.steps { display: flex; align-items: center; gap: var(--space-lg); }
.steps button { width: var(--size-touch); height: var(--size-touch); padding: 0; }
.next-icon { rotate: 180deg; }
#Prev:dir(rtl) og-icon { rotate: 180deg; }
#Next:dir(rtl) .next-icon { rotate: 0deg; }
.count { min-width: var(--size-xl); text-align: center; white-space: nowrap; font-variant-numeric: tabular-nums; }
.side { display: flex; flex-direction: column; gap: var(--space-lg); min-width: 0; }
.stats { display: flex; flex-direction: column; gap: var(--space-sm); }
[part="RacingGarage.row"] { display: flex; align-items: center; }
.actions { display: flex; gap: var(--space-sm); justify-content: flex-end; flex-wrap: wrap; }
.actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); gap: var(--space-xs); }
.reason { white-space: nowrap; }
/* The strip scrolls inside the garage's width and never widens it. */
.tiles-scroll { width: stretch; min-width: 0; flex: none; contain: inline-size; }
.tiles { display: flex; gap: var(--space-sm); padding: var(--space-xxs); }
[part="RacingGarage.tile"] { flex: none; width: calc(var(--size-xxl) * 1.7); min-height: var(--size-xl); padding: var(--space-xs) var(--space-sm); }
[part="RacingGarage.tile"] .face { display: flex; flex-direction: column; width: stretch; align-items: flex-start; gap: var(--space-xxs); }
.name { width: stretch; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.tag { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; max-width: stretch; padding: 0 var(--space-xs); }
.empty { display: flex; flex-direction: column; flex: 1; align-items: center; justify-content: center; gap: var(--space-md); }
.empty-text { margin: 0; }
@media (pointer: coarse) { .key { display: none; } }
/* A narrow panel stacks the shown car over the ratings and the action, the
   tiles along the foot under the thumb. The frame's own gap follows the
   viewport: a container cannot query itself. */
@media (width < 600px) { [part="RacingGarage.frame"] { gap: var(--space-sm); } }
@container (width < 600px) {
  .body { flex-direction: column; align-items: center; justify-content: flex-start; gap: var(--space-sm); overflow-y: auto; }
  .show { width: stretch; }
  .art, .well { aspect-ratio: 3.2; }
  .side { width: stretch; align-items: center; gap: var(--space-sm); }
  .actions { width: stretch; justify-content: center; }
}
</style>
