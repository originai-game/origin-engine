<meta name="component" content="ExplorePage">

<!-- Exploration HUD over a scene: the compass across the top, the tracked
     quest at the top right, the party down the left edge. -->
<div id="Page">
  <img id="Scene" src={$exploreScene} alt="">
  <div id="Compass"><RpgCompass heading={$compassHeading} marks={$compassMarks} /></div>
  <div id="Tracker"><RpgQuestTracker name={$trackName} steps={$trackSteps} distance={$trackDistance} /></div>
</div>

<style>
#Page { position: relative; width: stretch; height: stretch; overflow: hidden; }
#Scene { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
#Compass { position: absolute; left: 31%; top: var(--space-md); width: 32%; }
#Tracker { position: absolute; left: calc(100% - var(--size-xxl) * 4.6 - var(--space-lg)); top: var(--space-xl); width: calc(var(--size-xxl) * 4.6); }
@media (width < 600px) {
  #Compass { left: 5%; width: 90%; }
  #Tracker { left: var(--space-sm); top: calc(var(--size-lg) + var(--space-lg)); width: calc(100% - var(--space-sm) * 2); }
}
</style>
