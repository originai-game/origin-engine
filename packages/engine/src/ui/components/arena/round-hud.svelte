<meta name="component" content="ArenaRoundHud">
<meta name="strings" content="./arena.en.ogt">
<meta name="parts" content="ArenaRoundHud.frame ArenaRoundHud.team ArenaRoundHud.score ArenaRoundHud.round ArenaRoundHud.banner ArenaRoundHud.title ArenaRoundHud.badge ArenaRoundHud.plate ArenaRoundHud.figure ArenaRoundHud.caption">
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
<div id="ArenaRoundHud" part="ArenaRoundHud.frame" class="bare" data-layout="stack" data-gap="wide" role="group" aria-label={t("arena.round.name")}>
  <div class="bar" data-layout="row" data-gap="hair">
    <div part="ArenaRoundHud.team ArenaRoundHud.plate" class="team a" data-layout="row" data-local={localTeam == 'a'} role="group" aria-label={nameA != '' ? nameA : t("arena.team.a")}>
      <span class="name">{nameA != '' ? nameA : t("arena.team.a")}</span>
      {#if localTeam == 'a'}<span part="ArenaRoundHud.badge" class="you">{t("arena.you")}</span>{/if}
      <span part="ArenaRoundHud.score ArenaRoundHud.figure" class="score" data-team="a" aria-description={t("arena.score.teamScore")}>{formatNumber(scoreA)}</span>
    </div>
    <div part="ArenaRoundHud.round ArenaRoundHud.plate" class="round" data-layout="stack" data-gap="hair">
      <span class="round-label" part="ArenaRoundHud.caption">{rounds > 0 ? t("arena.round.roundOf", { round: round, rounds: rounds }) : t("arena.round.round", { round: round })}</span>
      {#if timeLeft >= 0}<div class="clock"><HudTimer seconds={timeLeft < 0 ? 0 : timeLeft} total={timeTotal} countdown={true} warnAt={10} paused={false} label="" /></div>{/if}
    </div>
    <div part="ArenaRoundHud.team ArenaRoundHud.plate" class="team b" data-layout="row" data-local={localTeam == 'b'} role="group" aria-label={nameB != '' ? nameB : t("arena.team.b")}>
      <span part="ArenaRoundHud.score ArenaRoundHud.figure" class="score" data-team="b" aria-description={t("arena.score.teamScore")}>{formatNumber(scoreB)}</span>
      {#if localTeam == 'b'}<span part="ArenaRoundHud.badge" class="you">{t("arena.you")}</span>{/if}
      <span class="name">{nameB != '' ? nameB : t("arena.team.b")}</span>
    </div>
  </div>
  <div class="call" data-layout="row" role="status" aria-live="assertive" aria-atomic="true">
    {#if outcome == 'won' || outcome == 'lost' || outcome == 'draw'}
    {#key at}
      <div part="ArenaRoundHud.banner ArenaRoundHud.plate" class="banner" data-layout="stack" data-gap="hair" data-motion="stamp" data-outcome={outcome}>
        <span part="ArenaRoundHud.title" class="outcome">{outcome == 'won' ? t("arena.round.won") : (outcome == 'lost' ? t("arena.round.lost") : (outcome == 'draw' ? t("arena.round.draw") : ''))}</span>
        {#if detail != ''}<span class="detail" part="ArenaRoundHud.caption">{detail}</span>{/if}
      </div>
    {/key}
    {/if}
  </div>
</div>

<style>
/* The round HUD over the match: each team's plate of smoked glass with
   its colour along its foot and its score large in that colour, the
   round and its clock between them; a round's verdict in the result's
   lit fill over a rule, held to the screen's width. */
:host { inline-size: stretch; min-inline-size: 0; align-items: center; pointer-events: none; color: var(--hud-ink); --layout-gap-wide: var(--space-xl); }
.bar { max-width: stretch; min-width: 0; align-items: stretch; justify-content: center; }
[part="ArenaRoundHud.team ArenaRoundHud.plate"] { flex: 0 1 auto; min-width: 0; padding: var(--space-xs) var(--space-lg); border: 0; border-radius: 0; background: var(--hud-plate-fill); box-shadow: var(--party-team-a-plate); }
.team.a { justify-content: flex-end; clip-path: polygon(var(--party-cut) 0, 100% 0, 100% 100%, 0 100%, 0 var(--party-cut)); }
.team.b { justify-content: flex-start; box-shadow: var(--party-team-b-plate); clip-path: polygon(0 0, calc(100% - var(--party-cut)) 0, 100% var(--party-cut), 100% 100%, 0 100%); }
.name { min-width: 0; font-size: var(--hud-size-sm); line-height: 1.15; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: none; overflow-wrap: break-word; }
.you { flex: none; padding: 0 var(--space-xs); border: 0; border-radius: 0; font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); text-shadow: none; clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); white-space: nowrap; color: var(--party-go-ink); background: var(--hud-accent); box-shadow: none; }
[part="ArenaRoundHud.score ArenaRoundHud.figure"] { flex: none; min-width: var(--size-md); text-align: center; white-space: nowrap; font-size: var(--party-size-figure); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--result-team-a-ink); filter: var(--result-team-a-glow); text-shadow: none; }
[part="ArenaRoundHud.score ArenaRoundHud.figure"][data-team=b] { color: var(--result-team-b-ink); filter: var(--result-team-b-glow); }
[part="ArenaRoundHud.round ArenaRoundHud.plate"] { flex: none; align-items: center; justify-content: center; padding: var(--space-xxs) var(--space-lg); border: 0; border-radius: 0; background: var(--hud-tag-fill); box-shadow: var(--hud-plate-rim); }
.round-label { font-size: var(--hud-label-size); line-height: 1.3; font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-shadow: none; }
.call { width: stretch; justify-content: center; container-type: inline-size; }
[part="ArenaRoundHud.banner ArenaRoundHud.plate"] { align-items: center; max-width: stretch; padding: 0; border: 0; border-radius: 0; background: none; box-shadow: none; --layout-gap-hair: var(--space-xs); }
.outcome { max-width: stretch; text-align: center; font-size: var(--party-size-call); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-verdict-tracking); text-transform: var(--hud-label-transform); overflow-wrap: break-word; color: transparent; background: var(--result-won-fill); background-clip: var(--hud-text-clip); filter: var(--result-won-glow); text-shadow: none; }
[data-outcome=lost] .outcome { background: var(--result-lost-fill); background-clip: var(--hud-text-clip); filter: var(--result-lost-glow); }
.detail { max-width: stretch; text-align: center; font-size: var(--hud-size-sm); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: var(--hud-shadow); overflow-wrap: break-word; }
/* The HUD spans the screen, so the screen's width decides. A phone keeps
   the scores and the round, and gives each team's name a line of its own
   over its score, so the name has the plate's whole width. */
@media (width < 600px) {
  :host { --layout-gap-wide: var(--space-lg); }
  /* The bar never outgrows the screen: the round and its clock take a
     line over the two teams, which share the width under it, so a name
     and a score keep whole at any text size. */
  .bar { width: stretch; flex-wrap: wrap; }
  [part="ArenaRoundHud.team ArenaRoundHud.plate"] { flex: 1 1 0; flex-wrap: wrap; align-content: center; --layout-gap: var(--space-xs); padding: var(--space-xxs) var(--space-sm); }
  .name { flex: 1 1 100%; font-size: var(--hud-size-xs); }
  .team.b .name { order: -1; }
  .team.a .name { text-align: end; }
  .team.b .name { text-align: start; }
  [part="ArenaRoundHud.score ArenaRoundHud.figure"] { font-size: var(--party-size-figure-compact); }
  [part="ArenaRoundHud.round ArenaRoundHud.plate"] { order: -1; flex: 1 1 100%; flex-direction: row; flex-wrap: wrap; min-width: 0; padding: var(--space-xxs) var(--space-sm); --layout-gap-hair: var(--space-sm); }
  .round-label { white-space: normal; text-align: center; }
}
</style>
