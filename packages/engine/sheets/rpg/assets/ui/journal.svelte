<meta name="component" content="JournalPage">

<!-- Desktop: the compass over the map; the quest log, the map with the place
     under it and the quest detail as three columns; the conversation across
     the foot. A phone: the party, the turn order, the map, the place and the
     quest log, fading out at the fold. -->
<div id="Page" class="rpg-ground">
  <div id="Compass"><Compass heading={$compassHeading} pois={$compassPois} units="m" /></div>
  <div id="Party"><RpgParty members={$partyMembers} selected={$partySelected} /></div>
  <div id="Turns"><RpgTurnOrder round={$turnRound} turns={$turns} /></div>
  <div id="Middle">
    <div id="Map"><RpgWorldMap map={$mapTexture} markers={$mapMarkers} selected={$mapSelected} legend={$mapLegend} here={$mapHere} /></div>
    <div id="Place"><RpgPlaceCard name={$placeName} about={$placeAbout} picture={$placePicture} marked={false} canTravel={true} /></div>
  </div>
  <div id="Log"><RpgQuestLog title={$logTitle} groups={$logGroups} quests={$logQuests} selected={$logSelected} /></div>
  <div id="Detail"><RpgQuestDetail name={$questName} summary={$questSummary} objectives={$questObjectives} rewards={$questRewards} scenery={$questScenery} label={$questLabel} tracked={true} canTrack={true} canLocate={true} /></div>
  <div id="Talk"><RpgConversation speaker={$talkSpeaker} portrait={$talkPortrait} line={$talkLine} replies={$talkReplies} selected={$talkSelected} /></div>
  <div id="Fade" class="rpg-fold"></div>
</div>

<style>
#Page { display: grid; width: stretch; height: stretch; grid-template-columns: 22% 1fr 27%; grid-template-rows: auto minmax(0, 3.7fr) minmax(0, 1fr); grid-template-areas: ". compass ." "log middle detail" "talk talk talk"; column-gap: var(--space-md); row-gap: var(--space-xs); padding: 0 var(--space-lg) var(--space-lg); }
#Compass { grid-area: compass; width: 72%; justify-self: center; }
#Log { grid-area: log; height: stretch; min-height: 0; }
#Middle { grid-area: middle; display: flex; flex-direction: column; height: stretch; min-height: 0; gap: var(--space-sm); }
#Map { width: stretch; flex: 1; min-height: 0; }
#Place { width: stretch; height: auto; flex: none; }
#Detail { grid-area: detail; height: stretch; min-height: 0; }
#Talk { grid-area: talk; height: stretch; min-height: 0; }
#Party, #Turns, #Fade { display: none; }
#Page:size(compact) { position: relative; display: flex; flex-direction: column; justify-content: flex-start; gap: var(--space-md); overflow: hidden; margin-top: calc(env(safe-area-inset-top) * -1); height: calc(100% + env(safe-area-inset-top)); padding: calc(env(safe-area-inset-top) + var(--space-sm)) var(--space-sm) 0; }
#Compass:size(compact), #Detail:size(compact), #Talk:size(compact) { display: none; }
#Party:size(compact), #Turns:size(compact) { display: block; width: stretch; flex: none; }
#Middle:size(compact) { width: stretch; flex: none; gap: var(--space-md); }
#Map:size(compact) { height: calc(var(--size-xxl) * 2.3); flex: none; }
#Place:size(compact) { height: calc(var(--size-xxl) * 2.25); flex: none; }
#Log:size(compact) { width: stretch; height: auto; flex: none; }
#Fade:size(compact) { display: block; position: absolute; left: 0; right: 0; bottom: 0; height: calc(var(--size-xxl) * 1.5); pointer-events: none; }
#Page:size(short) { grid-template-columns: 25% 1fr 25%; grid-template-rows: minmax(0, 1fr); grid-template-areas: "log middle detail"; column-gap: var(--space-sm); padding: var(--space-sm) var(--space-md); }
#Compass:size(short), #Talk:size(short) { display: none; }
#Place:size(short) { height: 56%; }
</style>
