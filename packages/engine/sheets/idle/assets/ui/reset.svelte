<meta name="component" content="IdleResetPage">

<!-- The stress page: prestige asking the player to confirm beside the
     welcome-back panel, then both in their long forms (long names, huge
     numbers, many rows, an unmet requirement). On a phone they stack. -->
<scroll id="Page">
  <div id="Grid">
    <div class="slot"><IdlePrestige title={$prestige_confirming.title} currency={$prestige_confirming.currency} icon={$prestige_confirming.icon} held={$prestige_confirming.held} gain={$prestige_confirming.gain} rows={$prestige_confirming_rows} requirement={$prestige_confirming.requirement} progress={$prestige_confirming.progress} progressText={$prestige_confirming.progressText} met={$prestige_confirming.met} confirming={$prestige_confirming.confirming} /></div>
    <div class="slot"><IdleOffline title={$offline_default.title} away={$offline_default.away} seconds={$offline_default.seconds} earned={$offline_default_earned} canDouble={$offline_default.canDouble} doubleText={$offline_default.doubleText} /></div>
    <div class="slot"><IdlePrestige title={$prestige_overflow.title} currency={$prestige_overflow.currency} icon={$prestige_overflow.icon} held={$prestige_overflow.held} gain={$prestige_overflow.gain} rows={$prestige_overflow_rows} requirement={$prestige_overflow.requirement} progress={$prestige_overflow.progress} progressText={$prestige_overflow.progressText} met={$prestige_overflow.met} confirming={$prestige_overflow.confirming} /></div>
    <div class="slot"><IdleOffline title={$offline_overflow.title} away={$offline_overflow.away} seconds={$offline_overflow.seconds} earned={$offline_overflow_earned} canDouble={$offline_overflow.canDouble} doubleText={$offline_overflow.doubleText} /></div>
  </div>
</scroll>

<style>
#Page { width: stretch; height: stretch; }
#Grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--space-xl); padding: var(--space-xl); }
.slot { display: flex; min-width: 0; justify-content: center; align-items: flex-start; }
@media (width < 600px) {
  #Grid { grid-template-columns: minmax(0, 1fr); gap: var(--space-md); padding: var(--space-sm); }
}
</style>
