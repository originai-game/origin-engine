<meta name="component" content="ExplorePage">

<!-- Exploration HUD over a scene, as on the board: the compass at the top,
     the tracked quest top right, the party down the left edge, the player's
     vitals bottom left, the action bar bottom centre, the minimap bottom
     right. -->
<div id="Page">
  <img id="Scene" src={$exploreScene} alt="">
  <div id="Hud">
      <div id="Party">
        {#each $partyMembers as m (m.id)}
          <RpgVitals name={m.name} portrait={m.portrait} level={m.level} health={m.health} healthMax={m.healthMax} mana={m.mana} manaMax={m.manaMax} plate="compact" down={m.down} />
        {/each}
      </div>
      <div id="Compass"><RpgCompass heading={$compassHeading} marks={$compassMarks} /></div>
      <div id="Tracker"><RpgQuestTracker name={$trackName} steps={$trackSteps} distance={$trackDistance} /></div>
      <div id="Player"><RpgVitals name={$playerName} portrait={$playerPortrait} level={$playerLevel} health={$playerHealth} healthMax={$playerHealthMax} mana={$playerMana} manaMax={$playerManaMax} plate="large" down={false} /></div>
      <div id="Bar"><RpgActionBar skills={$skills} items={$quickItems} /></div>
      <div id="Map" class="rpg-minimap"><Minimap texture={$mapTexture} u0={0.25} v0={0.15} u1={0.6} v1={0.65} size={170} markers={$minimapMarkers} label="" /></div>
  </div>
</div>

<style>
#Page { position: relative; width: stretch; height: stretch; overflow: hidden; }
#Scene { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
#Hud { position: relative; display: grid; width: stretch; height: stretch; grid-template-columns: auto minmax(0, 1fr) calc(var(--size-xxl) * 1.6) auto; grid-template-rows: auto 1fr auto; grid-template-areas: "party compass tracker tracker" "party . . ." "player bar bar map"; column-gap: var(--space-lg); row-gap: var(--space-md); padding: var(--space-md) var(--space-lg) var(--space-lg); }
#Party { grid-area: party; display: flex; flex-direction: column; gap: var(--space-sm); align-self: start; }
#Compass { grid-area: compass; width: stretch; padding: 0 var(--space-lg); }
#Tracker { grid-area: tracker; }
#Player { grid-area: player; align-self: end; }
#Bar { grid-area: bar; align-self: end; justify-self: center; padding-bottom: var(--space-lg); }
#Map { grid-area: map; align-self: end; justify-self: end; }
@media (width < 600px) {
  #Hud { grid-template-columns: minmax(0, 1fr) auto; grid-template-rows: auto auto auto 1fr auto auto; grid-template-areas: "compass compass" "tracker tracker" "party map" "party ." "bar bar" "player player"; gap: var(--space-sm); padding: var(--space-sm); }
  #Compass { width: stretch; }
  #Tracker { grid-area: tracker; }
  #Map { align-self: start; }
  #Bar { padding-bottom: var(--space-md); }
}
@media (height < 500px) {
  #Hud { grid-template-columns: auto minmax(0, 1fr) auto; grid-template-rows: auto 1fr auto; grid-template-areas: "party compass tracker" "party . ." "party bar map"; column-gap: var(--space-md); row-gap: var(--space-xs); padding: var(--space-sm) var(--space-md); }
  #Player { display: none; }
  #Party { gap: var(--space-xs); }
  #Bar { padding-bottom: 0; }
}
</style>
