<meta name="component" content="RpgPlaceCard">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgPlaceCard.frame RpgPlaceCard.name RpgPlaceCard.headline RpgPlaceCard.caption">
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
<section id="RpgPlaceCard" part="RpgPlaceCard.frame" data-layout="shell" data-gap="none" aria-label={name}>
  <div class="content" data-layout="row" data-gap="loose" data-fill>
  {#if picture}<img class="picture" src={picture} alt="">{/if}
  <div class="side" data-layout="switcher" data-gap="loose" data-fill>
    <div class="text" data-layout="stack" data-gap="tight" data-fill>
      <h3 part="RpgPlaceCard.name RpgPlaceCard.headline">{name}</h3>
      <div class="rpg-rule" role="separator"></div>
      {#if about}<p class="about" part="RpgPlaceCard.caption">{about}</p>{/if}
    </div>
    <div class="actions" data-layout="stack" data-fill>
      <button id="Mark" class="act secondary" data-glyph="mark" data-marked={marked} aria-pressed={marked} onclick={mark}><div class="face" data-layout="row"><og-icon class="act-mark" icon="flag"></og-icon><span class="act-label">{marked ? t("rpg.place.unmark") : t("rpg.place.mark")}</span></div></button>
      <button id="Travel" class="act secondary" data-glyph="travel" disabled={!canTravel} onclick={travel}><div class="face" data-layout="row"><og-icon class="act-mark" icon="map"></og-icon><span class="act-label">{t("rpg.place.travel")}</span></div></button>
    </div>
  </div>
  </div>
</section>

<style>
.picture { width: 34%; height: stretch; object-fit: cover; flex: none; }
.side { align-items: center; }
[part~="RpgPlaceCard.name"], .about { margin: 0; }
.act { min-height: var(--size-touch); }
.act .face { justify-content: center; }
.act-mark { width: var(--space-lg); height: var(--space-lg); flex: none; }
@container (width < 560px) {
  .picture { width: 48%; }
  .text { align-items: center; text-align: center; }
  .about { display: none; }
}
@container (width < 360px) { .picture { width: 36%; } }
</style>
