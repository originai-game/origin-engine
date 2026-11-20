<meta name="component" content="SongsPage">

<!-- The normal page, choosing what to play, laid out as the approved
     mockup: the song list and the selected song's details straight over the
     stage, the selected song's charts under them. The art is sample art of
     the sheet only. -->
<div id="Page">
  <img id="Stage" src={$selectStage} alt="">
  <div id="Shade"></div>
  <div id="Songs"><RhythmSongSelect title="Neon Festival" songs={$pageSongs} selected="s2" previewing="s2" status="" /></div>
  <div id="Charts"><RhythmDifficulty title={$difficulty_default.title} levels={$difficulty_default_levels} selected={$difficulty_default.selected} /></div>
</div>

<style>
#Page { position: relative; display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-lg); padding: calc(env(-og-hud-inset-top) + var(--space-xl)) calc(env(-og-hud-inset-right) + var(--space-xl) * 2) calc(env(-og-hud-inset-bottom) + var(--space-xl)) calc(env(-og-hud-inset-left) + var(--space-xl) * 2); }
#Stage { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; }
#Shade { position: absolute; inset: 0; background: linear-gradient(90deg, rgb(4 6 12 / 0.8), rgb(4 6 12 / 0.5) 50%, rgb(4 6 12 / 0.6)), linear-gradient(0deg, rgb(4 6 12 / 0.8), transparent 30%); }
#Songs { position: relative; flex: 1; min-height: 0; width: stretch; }
#Charts { position: relative; flex: none; width: stretch; }
#Page:size(compact) { gap: var(--space-sm); padding: calc(env(-og-hud-inset-top) + var(--space-sm)) calc(env(-og-hud-inset-right) + var(--space-sm)) calc(env(-og-hud-inset-bottom) + var(--space-sm)) calc(env(-og-hud-inset-left) + var(--space-sm)); }
#Shade:size(compact) { background: linear-gradient(180deg, rgb(4 6 12 / 0.55), rgb(4 6 12 / 0.85) 40%, rgb(4 6 12 / 0.92)); }
</style>
