<meta name="component" content="GeneralSeasonPage">

<!-- The shop family's season page: the pass with premium not owned (its
     tiers Locked, the free tier ready to claim), a chest before it is
     opened and after. A phone shows the pass over the revealed chest. -->
<div id="Page" class="general-ground">
  <div id="Pass"><PassTrack title={$blank} season={$passSeason} refresh={$passRefresh} level={$passLevel} progress={$passProgress} progressText={$passProgressText} premiumUnlocked={$no} upgradeLabel={$passUpgrade} rewards={$passRewards} loading={$no} error={$blank} disabled={$no} /></div>
  <div id="Sealed"><Reveal title={$revealTitle} kicker={$revealKicker} body={$revealBody} icon={$revealIcon} state={$revealSealed} rewards={$none} canSkip={$yes} continueLabel={$blank} disabled={$no} error={$blank} /></div>
  <div id="Revealed"><Reveal title={$revealTitle} kicker={$revealFound} body={$blank} icon={$revealIcon} state={$revealRevealed} rewards={$revealRewards} canSkip={$yes} continueLabel={$blank} disabled={$no} error={$blank} /></div>
</div>

<style>
#Page { display: grid; width: stretch; height: stretch; grid-template-columns: minmax(0, 1.7fr) minmax(0, 1fr); grid-template-rows: minmax(0, 1fr) minmax(0, 1fr); grid-template-areas: "pass sealed" "pass revealed"; gap: var(--space-lg); padding: var(--space-lg); }
#Pass { grid-area: pass; height: stretch; min-height: 0; }
#Sealed { grid-area: sealed; height: stretch; min-height: 0; }
#Revealed { grid-area: revealed; height: stretch; min-height: 0; }
#Page:size(compact) { display: flex; flex-direction: column; gap: var(--space-md); padding: var(--space-sm); overflow: hidden; }
#Pass:size(compact) { flex: none; }
#Sealed:size(compact) { display: none; }
#Revealed:size(compact) { flex: 1; min-height: 0; height: auto; }
</style>
