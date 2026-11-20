<meta name="component" content="ArenaScoreboard">
<meta name="strings" content="./arena.en.ogt">
<meta name="prop:title" content="string" data-doc="The board's heading; empty takes the kit's (Scoreboard).">
<meta name="prop:columns" content="string" data-doc="What each row shows: kda (kills, deaths, assists and score) or score (score only).">
<meta name="prop:nameA" content="string" data-doc="The first team's name; empty takes the kit's (Blue team).">
<meta name="prop:nameB" content="string" data-doc="The second team's name; empty takes the kit's (Red team).">
<meta name="prop:scoreA" content="string" data-doc="The first team's score as the game words it (rounds, points); empty hides it.">
<meta name="prop:scoreB" content="string" data-doc="The second team's score as the game words it; empty hides it.">
<meta name="prop:rows" content="list" data-doc="Every player in the game's order, keyed rows of { id, team, name, kills, deaths, assists, score, local, disconnected }: team a or b; the figures as the game counts them; local marks the local player's row (You, aria-current); disconnected dims the row and names it (Disconnected).">
<meta name="fixture:default" content='{"title":"","columns":"kda","nameA":"","nameB":"","scoreA":"7","scoreB":"5","rows":[{"id":"kai","team":"a","name":"Kai","kills":12,"deaths":4,"assists":6,"score":2840,"local":false,"disconnected":false},{"id":"you","team":"a","name":"Rhea","kills":9,"deaths":5,"assists":11,"score":2410,"local":true,"disconnected":false},{"id":"ana","team":"a","name":"Ana","kills":3,"deaths":7,"assists":14,"score":1720,"local":false,"disconnected":false},{"id":"rook","team":"b","name":"Rook","kills":10,"deaths":8,"assists":3,"score":2290,"local":false,"disconnected":false},{"id":"wren","team":"b","name":"Wren","kills":6,"deaths":9,"assists":7,"score":1650,"local":false,"disconnected":true},{"id":"quill","team":"b","name":"Quill","kills":4,"deaths":10,"assists":9,"score":1310,"local":false,"disconnected":false}]}'>
<meta name="fixture:score" content='{"title":"","columns":"score","nameA":"Iron Wolves","nameB":"Ember Court","scoreA":"2","scoreB":"3","rows":[{"id":"kai","team":"a","name":"Kai","kills":0,"deaths":0,"assists":0,"score":34,"local":false,"disconnected":false},{"id":"you","team":"a","name":"Rhea","kills":0,"deaths":0,"assists":0,"score":28,"local":true,"disconnected":false},{"id":"rook","team":"b","name":"Rook","kills":0,"deaths":0,"assists":0,"score":41,"local":false,"disconnected":false},{"id":"wren","team":"b","name":"Wren","kills":0,"deaths":0,"assists":0,"score":19,"local":false,"disconnected":false}]}'>
<meta name="fixture:empty" content='{"title":"","columns":"kda","nameA":"","nameB":"","scoreA":"","scoreB":"","rows":[]}'>
<meta name="fixture:disconnected" content='{"title":"","columns":"kda","nameA":"","nameB":"","scoreA":"0","scoreB":"1","rows":[{"id":"you","team":"a","name":"Rhea","kills":2,"deaths":1,"assists":0,"score":420,"local":true,"disconnected":false},{"id":"kai","team":"a","name":"Kai","kills":0,"deaths":0,"assists":0,"score":0,"local":false,"disconnected":true},{"id":"rook","team":"b","name":"Rook","kills":1,"deaths":2,"assists":0,"score":310,"local":false,"disconnected":true},{"id":"wren","team":"b","name":"Wren","kills":0,"deaths":0,"assists":1,"score":90,"local":false,"disconnected":true}]}'>
<meta name="fixture:overflow" content='{"title":"Championship Series · Grand Finals · Map 5 of 5 · Overtime","columns":"kda","nameA":"The Unbroken Wardens of the Ninefold Gate","nameB":"Ember Court Invitational Reserve","scoreA":"12","scoreB":"12","rows":[{"id":"a1","team":"a","name":"Maximilian Vandenberghe-Okonkwo","kills":128,"deaths":3,"assists":214,"score":184320,"local":false,"disconnected":false},{"id":"a2","team":"a","name":"Rhea","kills":31,"deaths":22,"assists":40,"score":9840,"local":true,"disconnected":false},{"id":"a3","team":"a","name":"Ana","kills":12,"deaths":30,"assists":51,"score":7210,"local":false,"disconnected":false},{"id":"a4","team":"a","name":"Kai","kills":20,"deaths":25,"assists":18,"score":6600,"local":false,"disconnected":true},{"id":"a5","team":"a","name":"Jo","kills":9,"deaths":28,"assists":33,"score":5120,"local":false,"disconnected":false},{"id":"b1","team":"b","name":"Abernathy-Whitcombe","kills":44,"deaths":19,"assists":20,"score":11250,"local":false,"disconnected":false},{"id":"b2","team":"b","name":"Wren","kills":25,"deaths":27,"assists":30,"score":8030,"local":false,"disconnected":false},{"id":"b3","team":"b","name":"Rook","kills":18,"deaths":24,"assists":22,"score":6900,"local":false,"disconnected":false},{"id":"b4","team":"b","name":"Moreau","kills":15,"deaths":29,"assists":26,"score":6010,"local":false,"disconnected":false},{"id":"b5","team":"b","name":"Laurent","kills":7,"deaths":33,"assists":41,"score":4880,"local":false,"disconnected":true}]}'>

<!--
  The team scoreboard: a section per team headed by its name and score,
  one row per player with K, D, A and score (or score alone), in the
  game's order. The local player's row is marked You and aria-current; a
  disconnected player's row is dimmed and says Disconnected. The game
  counts every figure; the board only shows them. Read only; it scrolls
  inside the height it is given.
-->
<section id="ArenaScoreboard" part="frame" variant="bare" layout="stack" fill gap="loose" aria-label={title != '' ? title : t("arena.score.title")}>
  <header class="band" part="band" layout="row"><h2 class="title" part="title" fill>{title != '' ? title : t("arena.score.title")}</h2></header>
  {#if rows.length}
    <div class="list" layout="scroll" fill>
      <div class="teams" layout="stack" gap="loose">
        <table part="plate" class="team" team="a" aria-label={(nameA != '' ? nameA : t("arena.team.a"))}>
          <thead>
            <tr part="heading" class="heading" team="a">
              <th class="team-head" scope="col"><div class="head" layout="row"><span class="team-name">{(nameA != '' ? nameA : t("arena.team.a"))}</span>{#if scoreA != ''}<span part="figure" class="team-score score" team="a" aria-description={t("arena.score.teamScore")}>{scoreA}</span>{/if}</div></th>
              {#if columns == 'kda'}
              <th class="num caption" part="caption" scope="col" aria-label={t("arena.score.killsName")}>{t("arena.score.kills")}</th>
              <th class="num caption" part="caption" scope="col" aria-label={t("arena.score.deathsName")}>{t("arena.score.deaths")}</th>
              <th class="num caption" part="caption" scope="col" aria-label={t("arena.score.assistsName")}>{t("arena.score.assists")}</th>
              {/if}
              <th class="total caption" part="caption" scope="col">{t("arena.score.score")}</th>
            </tr>
          </thead>
          <tbody>
            {#each rows as r (r.id)}
              {#if r.team == 'a'}
                <tr class="row" part="row" aria-current={r.local} class:local={r.local} class:disconnected={r.disconnected}>
                  <th class="who-cell" scope="row"><div class="who" layout="wrap"><span part="headline" class="name">{r.name}</span>{#if r.local}<span part="badge" class="you" data-tag="you">{t("arena.you")}</span>{/if}{#if r.disconnected}<span part="badge" class="off" data-tag="disconnected">{t("arena.score.disconnected")}</span>{/if}</div></th>
                  {#if columns == 'kda'}
                  <td part="figure" class="num stat">{formatNumber(r.kills)}</td>
                  <td part="figure" class="num stat">{formatNumber(r.deaths)}</td>
                  <td part="figure" class="num stat">{formatNumber(r.assists)}</td>
                  {/if}
                  <td part="figure" class="total stat">{formatNumber(r.score)}</td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
        <table part="plate" class="team" team="b" aria-label={(nameB != '' ? nameB : t("arena.team.b"))} class:team-b={true}>
          <thead>
            <tr part="heading" class="heading" team="b" class:team-b={true}>
              <th class="team-head" scope="col"><div class="head" layout="row"><span class="team-name">{(nameB != '' ? nameB : t("arena.team.b"))}</span>{#if scoreB != ''}<span part="figure" class="team-score score" team="b" aria-description={t("arena.score.teamScore")} class:team-b={true}>{scoreB}</span>{/if}</div></th>
              {#if columns == 'kda'}
              <th class="num caption" part="caption" scope="col" aria-label={t("arena.score.killsName")}>{t("arena.score.kills")}</th>
              <th class="num caption" part="caption" scope="col" aria-label={t("arena.score.deathsName")}>{t("arena.score.deaths")}</th>
              <th class="num caption" part="caption" scope="col" aria-label={t("arena.score.assistsName")}>{t("arena.score.assists")}</th>
              {/if}
              <th class="total caption" part="caption" scope="col">{t("arena.score.score")}</th>
            </tr>
          </thead>
          <tbody>
            {#each rows as r (r.id)}
              {#if r.team == 'b'}
                <tr class="row" part="row" aria-current={r.local} class:local={r.local} class:disconnected={r.disconnected}>
                  <th class="who-cell" scope="row"><div class="who" layout="wrap"><span part="headline" class="name">{r.name}</span>{#if r.local}<span part="badge" class="you" data-tag="you">{t("arena.you")}</span>{/if}{#if r.disconnected}<span part="badge" class="off" data-tag="disconnected">{t("arena.score.disconnected")}</span>{/if}</div></th>
                  {#if columns == 'kda'}
                  <td part="figure" class="num stat">{formatNumber(r.kills)}</td>
                  <td part="figure" class="num stat">{formatNumber(r.deaths)}</td>
                  <td part="figure" class="num stat">{formatNumber(r.assists)}</td>
                  {/if}
                  <td part="figure" class="total stat">{formatNumber(r.score)}</td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {:else}
    <p class="empty-text caption" part="caption" role="status">{t("arena.score.empty")}</p>
  {/if}
</section>

<style>
/* The team scoreboard over the match, bare: the title small and centred,
   each team a table headed by its name in its colour and its score set
   large, its rows on dark glass washed in the team's colour from an edge
   of it, the local player's row lit white, a disconnected player's row
   dimmed with its name kept and the tag beside it. Side by side where the
   board is wide, stacked where it is not. */
#ArenaScoreboard { inline-size: stretch; min-inline-size: 0; max-block-size: stretch; color: var(--hud-ink); }
.band { width: stretch; flex: none; justify-content: center; border: 0; padding: 0; min-height: 0; background: none; }
.title { margin: 0; flex: 1; min-width: 0; text-align: center; white-space: normal; overflow-wrap: anywhere; font-size: var(--hud-size-sm); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
.list { flex: 1; min-height: 0; overflow-y: auto; container-type: inline-size; -og-scroll-fade: var(--space-xl); }
.teams { width: stretch; display: grid; grid-template-columns: minmax(0, 1fr); gap: var(--space-lg); align-items: start; }
@container (width >= 900px) { .teams { grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); } }
.team { width: stretch; border-collapse: separate; border-spacing: 0 var(--space-xxs); background: none; box-shadow: none; border: 0; padding: 0; }
/* The second team's colour: its name, score, rule, row wash and edge. */
.team.team-b .heading { box-shadow: var(--result-team-b-rule); }
.team.team-b .team-name, .team.team-b .team-score { color: var(--result-team-b-ink); }
.team.team-b .team-score { filter: var(--result-team-b-glow); }
.team.team-b .row, .team.team-b .row:hover { background: var(--result-team-b-row); box-shadow: var(--result-team-b-edge); }
.team.team-b .row.local { background: var(--result-team-b-you-row); }
.team th, .team td { padding: var(--space-xs) var(--space-sm); font-weight: inherit; }
/* The head: the team's name in its colour, its score large, a rule of it under the captions. */
.heading { box-shadow: var(--result-team-a-rule); }
.heading th { vertical-align: bottom; }
.team-name { font-size: var(--hud-size-md); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--result-team-a-ink); }
.team-score { font-size: var(--hud-size-lg); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--result-team-a-ink); filter: var(--result-team-a-glow); }
.caption { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
/* A row: dark glass washed in the team's colour from its edge. */
.row { min-height: var(--size-md); background: var(--result-team-a-row); box-shadow: var(--result-team-a-edge); color: var(--hud-ink); }
.row:hover { background: var(--result-team-a-row); }
.row.local { background: var(--result-team-a-you-row); color: var(--chosen-ink); box-shadow: var(--result-team-a-edge); }
.row.disconnected { opacity: var(--down-opacity); }
.name { min-width: 0; text-align: start; overflow-wrap: anywhere; font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); color: inherit; }
.stat { font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: inherit; }
td.total { font-size: var(--hud-size-md); }
/* The name column takes what the figures leave; the figures size to their widest value. */
.team-head, .who-cell { width: 100%; text-align: start; }
.head, .who { min-width: 0; align-items: baseline; column-gap: var(--space-md); }
.who { align-items: center; column-gap: var(--space-xs); row-gap: var(--space-xxs); }
/* A long team name wraps beside its score; a player's tags sit beside the name and wrap under it. */
.team-name { min-width: 0; text-align: start; overflow-wrap: anywhere; }
.team-score { flex: none; min-width: var(--size-md); text-align: center; white-space: nowrap; }
.you, .off { flex: none; white-space: nowrap; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.off { color: var(--hud-ink-dim); }
.num, .total { text-align: end; white-space: nowrap; }
.num { min-width: var(--size-md); }
.empty-text { margin: 0; padding: var(--space-lg); text-align: center; }
.team th:size(compact), .team td:size(compact) { padding: var(--space-xxs) var(--space-xs); }
.team:size(compact), .team thead:size(compact), .team tbody:size(compact) { display: block; }
.team tr:size(compact) { display: flex; flex-wrap: wrap; align-items: center; width: stretch; margin-block-end: var(--space-xxs); }
.team-head:size(compact), .who-cell:size(compact) { flex: 1 1 100%; width: auto; min-width: 0; max-width: stretch; }
.num:size(compact) { flex: 1 1 0; min-width: 0; }
.total:size(compact) { flex: 2 1 0; min-width: 0; }
</style>
