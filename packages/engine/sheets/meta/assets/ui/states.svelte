<meta name="component" content="GeneralStatesPage">

<!-- The exceptional states: the shop failed, no achievements, quests
     loading and empty, today's reward already taken, a muted chat and no
     friends. A phone shows the failed shop and the taken reward; the phone
     cells of the other pages show the rest. The purchase page shows the
     confirmation's states. -->
<div id="Page" class="general-ground">
  <div id="ShopError"><Shop title={$blank} kicker={$blank} wallet={$none} tabs={$none} tab={$blank} offers={$none} prices={$none} selected={$blank} loading={$no} error={$shopError} /></div>
  <div id="AchEmpty"><Achievements title={$blank} summary={$blank} done={0} achievements={$none} /></div>
  <div id="QuestsLoading"><DailyQuests title={$blank} refresh={$blank} quests={$none} loading={$yes} /></div>
  <div id="QuestsEmpty"><DailyQuests title={$blank} refresh={$questsEmptyRefresh} quests={$none} loading={$no} /></div>
  <div id="RewardsTaken"><DailyRewards title={$blank} refresh={$rewardsClaimedRefresh} days={$rewardsClaimedDays} canClaim={false} /></div>
  <div id="ChatMuted"><Chat channels={$chatMutedChannels} channel={$world} messages={$chatMuted} draft={$blank} maxLength={200} canSend={false} notice={$chatMutedNotice} /></div>
  <div id="FriendsEmpty"><Friends title={$blank} requests={$none} friends={$none} selected={$blank} /></div>
</div>

<style>
#Page { display: grid; width: stretch; height: stretch; grid-template-columns: repeat(4, minmax(0, 1fr)); grid-template-rows: minmax(0, 1fr) minmax(0, 1fr); gap: var(--space-md); padding: var(--space-lg); }
#Page div { height: stretch; min-height: 0; }
#Page:size(compact) { display: flex; flex-direction: column; gap: var(--space-sm); padding: var(--space-sm); overflow: hidden; }
#Page div:size(compact) { width: stretch; height: auto; flex: 1; min-height: 0; }
#ShopError:size(compact) { flex: none; }
#AchEmpty:size(compact), #QuestsEmpty:size(compact), #QuestsLoading:size(compact), #ChatMuted:size(compact), #FriendsEmpty:size(compact) { display: none; }
</style>
