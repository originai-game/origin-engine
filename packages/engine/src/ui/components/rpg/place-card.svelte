<meta name="component" content="RpgPlaceCard">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgPlaceCard.frame RpgPlaceCard.name RpgPlaceCard.caption">
<meta name="prop:name" content="string" data-doc="The place's name.">
<meta name="prop:about" content="string" data-doc="A line or two about it; empty hides it.">
<meta name="prop:picture" content="number" data-doc="Its picture (og.load.texture); 0 hides it.">
<meta name="prop:marked" content="bool" data-doc="The player has marked it: the button offers Unmark.">
<meta name="prop:canTravel" content="bool" data-doc="Fast travel to it is possible: the button is enabled.">
<meta name="fixture:default" content='{"name":"Dawnhold Ruins","about":"Crumbling walls overlook the valley.","picture":0,"marked":false,"canTravel":true}'>
<meta name="fixture:marked" content='{"name":"Dawnhold Ruins","about":"Crumbling walls overlook the valley. The old bell still rings at dusk, though no one has climbed the tower in forty years.","picture":0,"marked":true,"canTravel":false}'>

<!--
  A place picked on the map, on dark glass: its picture at the start
  fading into the glass, its name in the HUD's title type over a gold
  rule, a line about it, and two buttons, Mark (a lit ghost plate) and
  Fast travel (the gold primary plate). Reports mark (marking or
  unmarking) and travel (fast travel). Narrow (under 560 px) the line
  about it is left out, so the name and the buttons keep their room.
-->
<section id="RpgPlaceCard" part="RpgPlaceCard.frame" class="bare" data-layout="row" data-gap="none" aria-label={name}>
  {#if picture}<div class="pic" data-layout="overlay"><img class="picture" data-anchor="fill" src={picture} alt=""><div class="fade" data-anchor="fill"></div></div>{/if}
  <div class="body" data-layout="stack" data-gap="tight" data-fill>
    <h3 class="name" part="RpgPlaceCard.name">{name}</h3>
    <div class="rule" role="separator"></div>
    {#if about}<p class="about" part="RpgPlaceCard.caption">{about}</p>{/if}
    <div class="actions" data-layout="row" data-gap="tight">
      <button id="Mark" class="act ghost" data-glyph="mark" data-marked={marked} aria-pressed={marked} onclick={mark}><div class="face" data-layout="row" data-gap="tight"><og-icon class="act-mark" icon="flag"></og-icon><span class="act-label">{marked ? t("rpg.place.unmark") : t("rpg.place.mark")}</span></div></button>
      <button id="Travel" class="act go" data-glyph="travel" disabled={!canTravel} onclick={travel}><div class="face" data-layout="row" data-gap="tight"><og-icon class="act-mark" icon="map"></og-icon><span class="act-label">{t("rpg.place.travel")}</span></div></button>
    </div>
  </div>
</section>

<style>
:host { inline-size: stretch; min-inline-size: 0; container-type: inline-size; color: var(--hud-ink); align-items: stretch; background: var(--map-card-fill-flat); box-shadow: var(--hud-plate-rim); --cut: calc(var(--hud-plate-cut) * 2); clip-path: polygon(var(--cut) 0, 100% 0, 100% calc(100% - var(--cut)), calc(100% - var(--cut)) 100%, 0 100%, 0 var(--cut)); }
.pic { flex: none; inline-size: 34%; min-block-size: stretch; }
.picture { width: 100%; height: 100%; object-fit: cover; }
.fade { background: var(--map-picture-fade); }
.body { min-inline-size: 0; justify-content: center; padding: var(--space-md) var(--space-lg); }
.name { margin: 0; max-inline-size: stretch; text-align: start; font-size: var(--map-title-size); line-height: 1.05; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); color: var(--hud-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.rule { inline-size: stretch; block-size: calc(2 * var(--font-step-px)); flex: none; background: var(--map-rule); }
/* The about line gives way first: at most two lines, fewer when the
   card is short, so the name and the buttons always show. */
.about { margin: 0; flex: 0 1 auto; min-block-size: 0; overflow: hidden; line-clamp: 2; text-align: start; font-size: var(--hud-size-sm); line-height: 1.45; color: var(--hud-ink-dim); }
.actions { flex: none; inline-size: stretch; padding-block-start: var(--space-xs); }
.act { flex: 1 1 auto; min-inline-size: 0; min-block-size: var(--size-touch); padding: 0 var(--space-sm); border: 0; border-radius: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); transition: var(--change-motion); }
.act .face { width: stretch; justify-content: center; align-items: center; }
.act-label { color: inherit; font-weight: inherit; white-space: nowrap; }
.act-mark { inline-size: var(--select-glyph-lg); block-size: var(--select-glyph-lg); flex: none; }
.ghost { color: var(--hud-ink); background: var(--hud-plate-fill); box-shadow: var(--map-ghost-rim); }
.ghost[aria-pressed=true] { color: var(--hud-gold); }
.go { color: var(--map-go-ink); background: var(--map-go-fill); box-shadow: var(--map-go-glow); }
.go:disabled { opacity: 0.45; }
.act:focus-visible { box-shadow: var(--map-focus-ring); }
@container (width < 560px) {
  .pic { inline-size: 26%; }
  .body { padding: var(--space-sm) var(--space-sm) var(--space-sm) var(--space-md); }
  .act { padding: 0 var(--space-xs); font-size: var(--map-act-compact-size); }
  .act-mark { display: none; }
  .name { font-size: var(--map-title-compact-size); }
  .about { display: none; }
}
</style>
