<meta name="component" content="RpgPlaceCard">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgPlaceCard.frame RpgPlaceCard.name">
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
<section id="RpgPlaceCard" part="RpgPlaceCard.frame" aria-label={name}>
  {#if picture}<img class="picture" src={picture} alt="">{/if}
  <div class="side">
    <div class="text">
      <h3 part="RpgPlaceCard.name">{name}</h3>
      <div class="rpg-rule" role="separator"></div>
      {#if about}<p class="about">{about}</p>{/if}
    </div>
    <div class="actions">
      <button id="Mark" class="act secondary" data-glyph="mark" data-marked={marked} aria-pressed={marked} onclick={mark}><div class="face"><og-icon class="act-mark" icon="flag"></og-icon><span class="act-label">{marked ? t("rpg.place.unmark") : t("rpg.place.mark")}</span></div></button>
      <button id="Travel" class="act secondary" data-glyph="travel" disabled={!canTravel} onclick={travel}><div class="face"><og-icon class="act-mark" icon="map"></og-icon><span class="act-label">{t("rpg.place.travel")}</span></div></button>
    </div>
  </div>
</section>

<style>
:host { display: flex; inline-size: stretch; block-size: stretch; gap: var(--space-md); container-type: inline-size; }
.picture { width: 34%; height: stretch; object-fit: cover; flex: none; }
.side { display: flex; flex: 1; gap: var(--space-md); align-items: center; min-width: 0; }
.text { display: flex; flex-direction: column; flex: 1; min-width: 0; gap: var(--space-xs); }
[part="RpgPlaceCard.name"], .about { margin: 0; }
.about { color: var(--ink-muted); line-height: var(--font-leading-normal); }
.actions { display: flex; flex-direction: column; gap: var(--space-sm); flex: none; width: 40%; }
.act { min-height: var(--size-touch); gap: var(--space-sm); }
.act .face { display: flex; align-items: center; justify-content: center; gap: var(--space-sm); }
.act-mark { width: var(--space-lg); height: var(--space-lg); flex: none; }
@container (width < 560px) {
  .picture { width: 48%; }
  .side { flex-direction: column; align-items: stretch; justify-content: center; }
  .text { align-items: center; text-align: center; }
  .about { display: none; }
  .actions { width: stretch; }
}
@container (width < 360px) { [part="RpgPlaceCard.frame"] { gap: var(--space-sm); } .picture { width: 36%; } .act .face { gap: var(--space-xs); } }
</style>
