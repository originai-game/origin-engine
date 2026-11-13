<meta name="component" content="SongsPage">

<!-- The normal page, choosing what to play: the song list with the selected
     song's details, and the charts of that song under it. -->
<div id="Page">
  <div id="Songs"><RhythmSongSelect title={$songSelect_default.title} songs={$songSelect_default_songs} selected={$songSelect_default.selected} previewing={$songSelect_default.previewing} status={$songSelect_default.status} /></div>
  <div id="Charts"><RhythmDifficulty title={$difficulty_default.title} levels={$difficulty_default_levels} selected={$difficulty_default.selected} /></div>
</div>

<style>
#Page { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-lg); padding: var(--space-xl); }
#Songs { flex: 1; min-height: 0; width: stretch; }
#Charts { flex: none; width: stretch; }
@media (width < 600px) {
  #Page { gap: var(--space-sm); padding: var(--space-sm); }
}
</style>
