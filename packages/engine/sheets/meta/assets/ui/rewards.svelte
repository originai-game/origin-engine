<meta name="component" content="GeneralRewardsPage">

<!-- The quests and rewards family's other states: achievements that are
     hidden, claimed, locked and in progress (on a phone too, where the
     progress page shows only the first rows), today's quests with long
     names and prizes, and a fortnight's calendar with milestones and
     today's reward waiting. -->
<div id="Page" class="general-ground">
  <div id="Achievements"><Achievements title={$blank} summary={$achLateSummary} done={$achLateDone} achievements={$achLate} /></div>
  <div id="Quests"><DailyQuests title={$questsLongTitle} refresh={$questsLongRefresh} quests={$questsLong} loading={$no} /></div>
  <div id="Rewards"><DailyRewards title={$blank} refresh={$blank} days={$rewardsFortnight} canClaim={true} /></div>
</div>

<style>
#Page { display: grid; width: stretch; height: stretch; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); grid-template-rows: minmax(0, 1fr) minmax(0, 1fr); grid-template-areas: "ach quests" "ach rewards"; gap: var(--space-lg); padding: var(--space-lg); }
#Achievements { grid-area: ach; height: stretch; min-height: 0; }
#Quests { grid-area: quests; height: stretch; min-height: 0; }
#Rewards { grid-area: rewards; height: stretch; min-height: 0; }
@media (width < 600px) {
  #Page { display: flex; flex-direction: column; gap: var(--space-md); padding: var(--space-sm); overflow: hidden; }
  #Quests { display: none; }
  #Achievements { width: stretch; height: 56%; flex: none; }
  #Rewards { width: stretch; flex: 1; min-height: 0; }
}
</style>
