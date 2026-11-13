<meta name="component" content="GeneralProgressPage">

<!-- Desktop: the achievements beside today's quests over the login
     calendar. A phone: the achievements over the quests (the calendar is
     on the states page's phone view). -->
<div id="Page" class="general-ground">
  <div id="Achievements"><Achievements title={$blank} summary={$achSummary} done={$achDone} achievements={$achList} /></div>
  <div id="Quests"><DailyQuests title={$blank} refresh={$questsRefresh} quests={$quests} loading={$no} /></div>
  <div id="Rewards"><DailyRewards title={$blank} refresh={$rewardsRefresh} days={$rewardsDays} canClaim={true} /></div>
</div>

<style>
#Page { display: grid; width: stretch; height: stretch; grid-template-columns: minmax(0, 1.25fr) minmax(0, 1fr); grid-template-rows: minmax(0, 1fr) minmax(0, 1fr); grid-template-areas: "ach quests" "ach rewards"; gap: var(--space-lg); padding: var(--space-lg); }
#Achievements { grid-area: ach; height: stretch; min-height: 0; }
#Quests { grid-area: quests; height: stretch; min-height: 0; }
#Rewards { grid-area: rewards; height: stretch; min-height: 0; }
@media (width < 600px) {
  #Page { display: flex; flex-direction: column; gap: var(--space-md); padding: var(--space-sm); overflow: hidden; }
  #Rewards { display: none; }
  #Achievements { width: stretch; height: 52%; flex: none; }
  #Quests { width: stretch; flex: 1; min-height: 0; }
}
</style>
