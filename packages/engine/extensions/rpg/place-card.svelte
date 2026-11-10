<meta name="component" content="RpgPlaceCard">
<meta name="strings" content="./rpg.en.ogt">
<meta name="prop:name" content="string" data-doc="The place's name.">
<meta name="prop:about" content="string" data-doc="A line or two about it; empty hides it.">
<meta name="prop:picture" content="number" data-doc="Its picture (og.load.texture); 0 hides it.">
<meta name="prop:marked" content="bool" data-doc="The player has marked it: the button offers Unmark.">
<meta name="prop:canTravel" content="bool" data-doc="Fast travel to it is possible: the button is enabled.">
<meta name="fixture:default" content='{"name":"Dawnhold Ruins","about":"Crumbling walls overlook the valley.","picture":0,"marked":false,"canTravel":true}'>

<!--
  A place picked on the map: its picture, its name and a line about it,
  and two buttons. Reports mark (marking or unmarking) and travel (fast
  travel). Wide, the text and the buttons stand side by side beside the
  picture; narrow (under 560 px), the name sits over the buttons and the
  line about it is left out.
-->
<section id="RpgPlaceCard" class="rpg-place-card" aria-label={name}>
  <img class="picture" src={picture} alt="" hidden={!picture}>
  <div class="side">
    <div class="text">
      <h3 class="place-name">{name}</h3>
      <div class="rpg-rule" role="separator"></div>
      <p class="about" hidden={!about}>{about}</p>
    </div>
    <div class="actions">
      <button id="Mark" class="act secondary" data-glyph="mark" data-marked={marked} onclick={mark}><div class="face"><div class="act-mark"></div><span class="act-label">{marked ? t("rpg.place.unmark") : t("rpg.place.mark")}</span></div></button>
      <button id="Travel" class="act secondary" data-glyph="travel" disabled={!canTravel} onclick={travel}><div class="face"><div class="act-mark"></div><span class="act-label">{t("rpg.place.travel")}</span></div></button>
    </div>
  </div>
</section>

<style>
.rpg-place-card { display: flex; width: stretch; height: stretch; gap: var(--space-md); container-type: inline-size; padding: var(--space-sm); }
.picture { width: 34%; height: stretch; object-fit: cover; flex: none; }
.side { display: flex; width: stretch; gap: var(--space-md); align-items: center; }
.text { display: flex; flex-direction: column; width: stretch; gap: var(--space-xs); }
.place-name, .about { margin: 0; }
.actions { display: flex; flex-direction: column; gap: var(--space-sm); flex: none; width: 40%; }
.act { width: stretch; min-height: var(--size-touch); gap: var(--space-sm); }
.act .face { display: flex; width: stretch; align-items: center; justify-content: center; gap: var(--space-sm); }
.act-mark { width: var(--space-lg); height: var(--space-lg); flex: none; }
@container (width < 560px) {
  .picture { width: 48%; }
  .side { flex-direction: column; align-items: stretch; justify-content: center; }
  .text { align-items: center; text-align: center; }
  .about { display: none; }
  .actions { width: stretch; }
}
</style>
