<meta name="component" content="DeliveryHud">
<meta name="strings" content="./hud.en.ogt">
<meta name="prop:shown" content="bool" data-doc="Whether the HUD is drawn; the game toggles it.">
<meta name="prop:seconds" content="number" data-doc="Seconds left to make the drop.">
<meta name="prop:total" content="number" data-doc="The delivery's whole time.">
<meta name="prop:parcel" content="string" data-doc="The parcel's name, on the tracker's plate.">
<meta name="prop:steps" content="list" data-doc="The delivery's steps, keyed rows of { id, text, done }.">
<meta name="prop:distance" content="string" data-doc="How far the drop is, as the game words it.">
<meta name="prop:condition" content="number" data-doc="The parcel's condition, 0 to 100.">
<meta name="prop:map" content="number" data-doc="The district's map texture.">
<meta name="prop:markers" content="list" data-doc="The minimap's markers, keyed rows of { id, x, y, tone, player }.">
<meta name="prop:district" content="string" data-doc="The district's name, under the minimap.">
<meta name="prop:hints" content="list" data-doc="The switches, keyed rows of { action, hint, hold }.">
<meta name="fixture:default" content='{"shown":true,"seconds":142,"total":180,"parcel":"Parcel 7C · Fragile","steps":[{"id":"collect","text":"Collect at Lantern Row","done":true},{"id":"deliver","text":"Deliver to Gate Nine","done":false}],"distance":"29 m","condition":96,"map":0,"markers":[{"id":"me","x":0.52,"y":0.89,"tone":"neutral","player":true},{"id":"drop","x":0.5,"y":0.32,"tone":"accent","player":false}],"district":"Lantern Row","hints":[{"action":"weather","hint":"Weather","hold":false},{"action":"season","hint":"Season","hold":false}]}'>

<!-- The courier's HUD: the district's minimap top left, the time left to
     make the drop top centre, the parcel and its steps top right, and the
     switches along the bottom. The lower third of the frame is left to the
     street. -->
<div id="Root">
{#if $shown}
<div id="Hud">
  <div id="Map"><Minimap texture={$map} u0={0} v0={0} u1={1} v1={1} size={168} markers={$markers} label={$district} /></div>
  <div id="Clock"><HudTimer seconds={$seconds} total={$total} countdown={true} warnAt={30} paused={false} label={t("hud.drop")} /></div>
  <div id="Parcel">
    <RpgQuestTracker name={$parcel} steps={$steps} distance={$distance} />
    <StatBar value={$condition} max={100} kind="stamina" label={t("hud.parcel")} showValue={true} valueText={$condition + " %"} />
  </div>
  <div id="Keys"><ActionHints hints={$hints} label={t("hud.switches")} /></div>
</div>
{/if}
</div>

<style>
#Root { position: fixed; inset: 0; pointer-events: none; }
#Hud { width: stretch; height: stretch; display: grid; grid-template-columns: auto minmax(0, 1fr) calc(var(--size-xxl) * 1.7); grid-template-rows: auto 1fr auto; grid-template-areas: "map clock parcel" ". . ." "keys keys keys"; padding: var(--space-lg); }
#Map { grid-area: map; align-self: start; }
#Clock { grid-area: clock; justify-self: center; align-self: start; }
#Parcel { grid-area: parcel; align-self: start; display: flex; flex-direction: column; gap: var(--space-sm); }
#Keys { grid-area: keys; justify-self: center; align-self: end; }
@media (width < 700px) {
  #Hud { grid-template-columns: auto minmax(0, 1fr); grid-template-areas: "map clock" "parcel parcel" "keys keys"; grid-template-rows: auto auto 1fr; gap: var(--space-sm); padding: var(--space-sm); }
  #Keys { display: none; }
}
</style>
