<meta name="component" content="ExplorePage">

<!-- Exploration HUD over a scene, as on the board: the compass at the top,
     the tracked quest top right, the party down the left edge, the player's
     vitals bottom left, the action bar bottom centre, the minimap bottom
     right. -->
<div id="Page">
  <img id="Scene" src={$exploreScene} alt="">
  <div id="Hud">
    <div id="Top">
      <div id="Party">
        {#each $partyMembers as m (m.id)}
          <RpgVitals name={m.name} portrait={m.portrait} level={m.level} health={m.health} healthMax={m.healthMax} mana={m.mana} manaMax={m.manaMax} plate="compact" down={m.down} />
        {/each}
      </div>
      <div id="Compass"><RpgCompass heading={$compassHeading} marks={$compassMarks} /></div>
      <div id="Tracker"><RpgQuestTracker name={$trackName} steps={$trackSteps} distance={$trackDistance} /></div>
    </div>
    <div id="Foot">
      <div id="Player"><RpgVitals name={$playerName} portrait={$playerPortrait} level={$playerLevel} health={$playerHealth} healthMax={$playerHealthMax} mana={$playerMana} manaMax={$playerManaMax} plate="large" down={false} /></div>
      <div id="Bar"><RpgActionBar skills={$skills} items={$quickItems} /></div>
      <div id="Map" class="rpg-minimap"><Minimap texture={$mapTexture} u0={0.25} v0={0.15} u1={0.6} v1={0.65} size={170} markers={$minimapMarkers} label="" /></div>
    </div>
  </div>
</div>

<style>
#Page { position: relative; width: stretch; height: stretch; overflow: hidden; }
#Scene { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
#Hud { position: relative; display: flex; flex-direction: column; justify-content: space-between; width: stretch; height: stretch; padding: var(--space-md) var(--space-lg) var(--space-lg); }
#Top { display: flex; width: stretch; align-items: flex-start; gap: var(--space-lg); }
#Party { display: flex; flex-direction: column; flex: none; width: calc(var(--size-xxl) * 3.2); gap: var(--space-sm); }
#Compass { flex: 1; min-width: 0; padding: 0 var(--space-xl); }
#Tracker { flex: none; width: calc(var(--size-xxl) * 4.6); }
#Foot { display: flex; width: stretch; align-items: flex-end; justify-content: space-between; gap: var(--space-lg); }
#Player { flex: none; width: calc(var(--size-xxl) * 4.2); }
#Bar { flex: none; padding-bottom: var(--space-lg); }
#Map { flex: none; }
@media (width < 600px) {
  #Hud { padding: var(--space-sm); }
  #Top { flex-wrap: wrap; gap: var(--space-sm); }
  #Compass { order: -1; width: 100%; flex: none; padding: 0; }
  #Party { width: fit-content; }
  #Tracker { order: -1; width: 100%; flex: none; }
  #Foot { flex-wrap: wrap; gap: var(--space-sm); }
  #Bar { order: -1; width: 100%; padding: 0; }
  #Player { width: calc(100% - 150px); }
}
</style>
