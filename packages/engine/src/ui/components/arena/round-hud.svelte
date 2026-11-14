<meta name="component" content="ArenaRoundHud">
<meta name="strings" content="./arena.en.ogt">
<meta name="parts" content="ArenaRoundHud.frame ArenaRoundHud.team ArenaRoundHud.score ArenaRoundHud.round ArenaRoundHud.banner ArenaRoundHud.title ArenaRoundHud.badge">
<meta name="prop:round" content="number" data-doc="The round being played, as the game counts it.">
<meta name="prop:rounds" content="number" data-doc="How many rounds the match has (Round 3 of 7); 0 hides the total.">
<meta name="prop:nameA" content="string" data-doc="The first team's name; empty takes the kit's (Blue team).">
<meta name="prop:nameB" content="string" data-doc="The second team's name; empty takes the kit's (Red team).">
<meta name="prop:scoreA" content="number" data-doc="The first team's score (rounds won) as the game counts it.">
<meta name="prop:scoreB" content="number" data-doc="The second team's score as the game counts it.">
<meta name="prop:localTeam" content="string" data-doc="The local player's team, a or b (its plate is marked You); empty is a spectator.">
<meta name="prop:timeLeft" content="number" data-default="-1" data-doc="Seconds left in the round as the game counts them (a HudTimer); negative hides it.">
<meta name="prop:timeTotal" content="number" data-doc="The round's whole time, for the timer's ring (0: no ring).">
<meta name="prop:outcome" content="string" data-default="none" data-doc="The round's end as the game decided it, for the local side: won, lost, draw, or none (no banner).">
<meta name="prop:at" content="number" data-doc="The banner's id: a new value replays its entrance.">
<meta name="prop:detail" content="string" data-doc="A line under the banner as the game words it (Flawless, Match point); empty hides it.">
<meta name="fixture:default" content='{"round":4,"rounds":7,"nameA":"","nameB":"","scoreA":2,"scoreB":1,"localTeam":"a","timeLeft":74,"timeTotal":120,"outcome":"none","at":0,"detail":""}'>
<meta name="fixture:won" content='{"round":5,"rounds":7,"nameA":"","nameB":"","scoreA":3,"scoreB":1,"localTeam":"a","timeLeft":-1,"timeTotal":0,"outcome":"won","at":1,"detail":"Match point"}'>
<meta name="fixture:lost" content='{"round":5,"rounds":7,"nameA":"","nameB":"","scoreA":2,"scoreB":3,"localTeam":"a","timeLeft":-1,"timeTotal":0,"outcome":"lost","at":2,"detail":""}'>
<meta name="fixture:draw" content='{"round":6,"rounds":0,"nameA":"","nameB":"","scoreA":3,"scoreB":3,"localTeam":"","timeLeft":0,"timeTotal":120,"outcome":"draw","at":3,"detail":"Time ran out"}'>
<meta name="fixture:critical" content='{"round":7,"rounds":7,"nameA":"","nameB":"","scoreA":3,"scoreB":3,"localTeam":"b","timeLeft":4.2,"timeTotal":120,"outcome":"none","at":0,"detail":""}'>
<meta name="fixture:overflow" content='{"round":128,"rounds":250,"nameA":"The Unbroken Wardens of the Ninefold Gate","nameB":"Ember Court Invitational Reserve","scoreA":1204,"scoreB":998,"localTeam":"a","timeLeft":3599,"timeTotal":3600,"outcome":"won","at":4,"detail":"Flawless victory in overtime after the defenders held both points to the final second"}'>

<!--
  The round HUD along the top of a match: each team's name and score on
  either side of the round and its timer (HudTimer, as the game counts
  it), the local team's plate marked You. When the game ends a round it
  sets outcome: a banner across the screen says Round won, Round lost or
  Round drawn (a status announced at once), its entrance replayed for a
  new id and skipped under reduced motion. The game counts the score and
  the time and decides the outcome. Read only.
-->
<div id="ArenaRoundHud" part="ArenaRoundHud.frame" class="bare" role="group" aria-label={t("arena.round.name")}>
  <div class="bar">
    <div part="ArenaRoundHud.team" class="team a" data-local={localTeam == 'a'} role="group" aria-label={nameA != '' ? nameA : t("arena.team.a")}>
      <span class="name">{nameA != '' ? nameA : t("arena.team.a")}</span>
      <span part="ArenaRoundHud.badge" class="you" hidden={localTeam != 'a'}>{t("arena.you")}</span>
      <span part="ArenaRoundHud.score" class="score" data-team="a" aria-description={t("arena.score.teamScore")}>{formatNumber(scoreA)}</span>
    </div>
    <div part="ArenaRoundHud.round" class="round">
      <span class="round-label">{rounds > 0 ? t("arena.round.roundOf", { round: round, rounds: rounds }) : t("arena.round.round", { round: round })}</span>
      <div class="clock" hidden={timeLeft < 0}><HudTimer seconds={timeLeft < 0 ? 0 : timeLeft} total={timeTotal} countdown={true} warnAt={10} paused={false} label="" /></div>
    </div>
    <div part="ArenaRoundHud.team" class="team b" data-local={localTeam == 'b'} role="group" aria-label={nameB != '' ? nameB : t("arena.team.b")}>
      <span part="ArenaRoundHud.score" class="score" data-team="b" aria-description={t("arena.score.teamScore")}>{formatNumber(scoreB)}</span>
      <span part="ArenaRoundHud.badge" class="you" hidden={localTeam != 'b'}>{t("arena.you")}</span>
      <span class="name">{nameB != '' ? nameB : t("arena.team.b")}</span>
    </div>
  </div>
  <div class="call" role="status" aria-live="assertive" aria-atomic="true">
    {#key at}
      <div part="ArenaRoundHud.banner" class="banner" hidden={!(outcome == 'won' || outcome == 'lost' || outcome == 'draw')} data-outcome={outcome}>
        <span part="ArenaRoundHud.title" class="outcome">{outcome == 'won' ? t("arena.round.won") : (outcome == 'lost' ? t("arena.round.lost") : (outcome == 'draw' ? t("arena.round.draw") : ''))}</span>
        <span class="detail" hidden={detail == ''}>{detail}</span>
      </div>
    {/key}
  </div>
</div>

<style>
[part="ArenaRoundHud.frame"][hidden], [part="ArenaRoundHud.frame"] [hidden] { display: none; }
[part="ArenaRoundHud.frame"] { display: flex; flex-direction: column; width: stretch; min-width: 0; align-items: center; gap: var(--space-xl); pointer-events: none; }
.bar { display: flex; max-width: stretch; min-width: 0; align-items: stretch; justify-content: center; gap: var(--space-xxs); }
[part="ArenaRoundHud.team"] { display: flex; flex: 0 1 auto; min-width: 0; align-items: center; gap: var(--space-sm); padding: var(--space-xs) var(--space-md); }
.team.a { justify-content: flex-end; }
.team.b { justify-content: flex-start; }
.name { min-width: 0; max-width: calc(var(--size-xxl) * 2.2); white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.you { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
[part="ArenaRoundHud.score"] { flex: none; min-width: var(--size-md); text-align: center; white-space: nowrap; font-variant-numeric: tabular-nums; }
[part="ArenaRoundHud.round"] { display: flex; flex-direction: column; flex: none; align-items: center; justify-content: center; gap: var(--space-xxs); padding: var(--space-xxs) var(--space-md); }
.round-label { white-space: nowrap; }
.call { display: flex; width: stretch; justify-content: center; }
[part="ArenaRoundHud.banner"] { display: flex; flex-direction: column; align-items: center; max-width: stretch; gap: var(--space-xxs); padding: var(--space-sm) var(--space-3xl); animation-name: arena-banner; animation-duration: var(--motion-duration-slow); animation-timing-function: var(--motion-easing-out); animation-fill-mode: both; }
.outcome { white-space: nowrap; }
.detail { text-align: center; max-width: stretch; }
@keyframes arena-banner { from { opacity: 0; scale: 1.3; } to { opacity: 1; scale: 1; } }
@media (prefers-reduced-motion: reduce) { [part="ArenaRoundHud.banner"] { animation-name: none; } }
/* The HUD spans the screen, so the screen's width decides. A phone keeps
   the scores and the round, and gives each team's name a line of its own
   over its score, so the name has the plate's whole width. */
@media (width < 600px) {
  [part="ArenaRoundHud.frame"] { gap: var(--space-lg); }
  /* The bar never outgrows the screen: the plates share the width, the round's words wrap. */
  .bar { width: stretch; }
  [part="ArenaRoundHud.team"] { flex: 1 1 0; flex-wrap: wrap; align-content: center; gap: var(--space-xxs) var(--space-xs); padding: var(--space-xxs) var(--space-xs); }
  .name { flex: 1 1 100%; max-width: none; }
  .team.b .name { order: -1; }
  .team.a .name { text-align: end; }
  .team.b .name { text-align: start; }
  [part="ArenaRoundHud.round"] { flex: 0 1 auto; min-width: 0; padding: var(--space-xxs) var(--space-xs); }
  .round-label { white-space: normal; text-align: center; }
  [part="ArenaRoundHud.banner"] { padding: var(--space-xs) var(--space-xl); }
}
</style>
