<meta name="component" content="PuzzlePlayPage">

<!-- In a level, over the game's map art (sample art of the sheet only): the
     goals and moves over the board, the board itself a stand-in the game
     would draw (eight rows of sample pieces, every row whole in the space
     between the goals bar and the tray), the booster tray under it with
     the hammer armed and one booster out. -->
<div id="Page">
  <img id="Art" src={$meadow} alt="">
  <div id="Hud"><PuzzleGoals counter={$goals_default.counter} moves={$goals_default.moves} seconds={$goals_default.seconds} low={$goals_default.low} score={$goals_default.score} progress={$goals_default.progress} stars={$goals_default_stars} goals={$goals_default_goals} /></div>
  <div id="Board" role="image" aria-label="Board">
    <div class="row"><div class="cell"><og-icon icon="apple"></og-icon></div><div class="cell"><og-icon icon="gem"></og-icon></div><div class="cell"><og-icon icon="leaf"></og-icon></div><div class="cell"><og-icon icon="drop"></og-icon></div><div class="cell"><og-icon icon="coin"></og-icon></div><div class="cell"><og-icon icon="apple"></og-icon></div><div class="cell"><og-icon icon="gem"></og-icon></div><div class="cell"><og-icon icon="leaf"></og-icon></div></div>
    <div class="row"><div class="cell"><og-icon icon="drop"></og-icon></div><div class="cell"><og-icon icon="coin"></og-icon></div><div class="cell"><og-icon icon="apple"></og-icon></div><div class="cell"><og-icon icon="gem"></og-icon></div><div class="cell"><og-icon icon="leaf"></og-icon></div><div class="cell"><og-icon icon="apple"></og-icon></div><div class="cell"><og-icon icon="gem"></og-icon></div><div class="cell"><og-icon icon="leaf"></og-icon></div></div>
    <div class="row"><div class="cell"><og-icon icon="gem"></og-icon></div><div class="cell"><og-icon icon="leaf"></og-icon></div><div class="cell"><og-icon icon="apple"></og-icon></div><div class="cell"><og-icon icon="gem"></og-icon></div><div class="cell"><og-icon icon="leaf"></og-icon></div><div class="cell"><og-icon icon="drop"></og-icon></div><div class="cell"><og-icon icon="coin"></og-icon></div><div class="cell"><og-icon icon="apple"></og-icon></div></div>
    <div class="row"><div class="cell"><og-icon icon="gem"></og-icon></div><div class="cell"><og-icon icon="leaf"></og-icon></div><div class="cell"><og-icon icon="drop"></og-icon></div><div class="cell"><og-icon icon="coin"></og-icon></div><div class="cell"><og-icon icon="apple"></og-icon></div><div class="cell"><og-icon icon="gem"></og-icon></div><div class="cell"><og-icon icon="leaf"></og-icon></div><div class="cell"><og-icon icon="apple"></og-icon></div></div>
    <div class="row"><div class="cell"><og-icon icon="coin"></og-icon></div><div class="cell"><og-icon icon="apple"></og-icon></div><div class="cell"><og-icon icon="gem"></og-icon></div><div class="cell"><og-icon icon="leaf"></og-icon></div><div class="cell"><og-icon icon="apple"></og-icon></div><div class="cell"><og-icon icon="gem"></og-icon></div><div class="cell"><og-icon icon="leaf"></og-icon></div><div class="cell"><og-icon icon="drop"></og-icon></div></div>
    <div class="row"><div class="cell"><og-icon icon="leaf"></og-icon></div><div class="cell"><og-icon icon="apple"></og-icon></div><div class="cell"><og-icon icon="gem"></og-icon></div><div class="cell"><og-icon icon="leaf"></og-icon></div><div class="cell"><og-icon icon="drop"></og-icon></div><div class="cell"><og-icon icon="coin"></og-icon></div><div class="cell"><og-icon icon="apple"></og-icon></div><div class="cell"><og-icon icon="gem"></og-icon></div></div>
    <div class="row"><div class="cell"><og-icon icon="leaf"></og-icon></div><div class="cell"><og-icon icon="drop"></og-icon></div><div class="cell"><og-icon icon="coin"></og-icon></div><div class="cell"><og-icon icon="apple"></og-icon></div><div class="cell"><og-icon icon="gem"></og-icon></div><div class="cell"><og-icon icon="leaf"></og-icon></div><div class="cell"><og-icon icon="apple"></og-icon></div><div class="cell"><og-icon icon="gem"></og-icon></div></div>
    <div class="row"><div class="cell"><og-icon icon="apple"></og-icon></div><div class="cell"><og-icon icon="gem"></og-icon></div><div class="cell"><og-icon icon="leaf"></og-icon></div><div class="cell"><og-icon icon="apple"></og-icon></div><div class="cell"><og-icon icon="gem"></og-icon></div><div class="cell"><og-icon icon="leaf"></og-icon></div><div class="cell"><og-icon icon="drop"></og-icon></div><div class="cell"><og-icon icon="coin"></og-icon></div></div>
  </div>
  <div id="Tray"><PuzzleBoosters armed="hammer" boosters={$boosters_default_boosters} /></div>
</div>

<style>
#Page { position: relative; display: flex; flex-direction: column; width: stretch; height: stretch; align-items: center; gap: var(--space-sm); padding: var(--space-md); }
#Art { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; }
#Hud, #Tray { width: 64%; flex: none; }
/* The board takes what the goals bar and the tray leave; its rows share
   that height, so none is cut. */
#Board { display: flex; flex-direction: column; width: 46%; flex: 1; min-height: 0; padding: var(--space-xs); background: rgb(14 30 12 / 0.72); box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.12); }
.row { display: flex; flex: 1; min-height: 0; }
.cell { display: flex; flex: 1; min-width: 0; min-height: 0; align-items: center; justify-content: center; box-shadow: inset 0 0 0 1px rgb(0 0 0 / 0.18); }
.cell og-icon { width: 78%; height: 78%; }
@media (width < 600px) {
  #Page { gap: var(--space-xs); padding: var(--space-xs); }
  #Hud, #Board, #Tray { width: stretch; }
}
</style>
