<meta name="component" content="ArenaScoreboard">
<meta name="strings" content="./arena.en.ogt">
<meta name="parts" content="ArenaScoreboard.frame ArenaScoreboard.band ArenaScoreboard.title ArenaScoreboard.team ArenaScoreboard.heading ArenaScoreboard.score ArenaScoreboard.row ArenaScoreboard.cell ArenaScoreboard.name ArenaScoreboard.badge">
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
<section id="ArenaScoreboard" part="ArenaScoreboard.frame" aria-label={title != '' ? title : t("arena.score.title")}>
  <header part="ArenaScoreboard.band"><h2 part="ArenaScoreboard.title">{title != '' ? title : t("arena.score.title")}</h2></header>
  {#if rows.length}
    <scroll class="list">
      <div class="teams">
        <table part="ArenaScoreboard.team" class="team" aria-label={(nameA != '' ? nameA : t("arena.team.a"))}>
          <thead>
            <tr part="ArenaScoreboard.heading" class="heading" data-team="a">
              <th class="team-head" scope="col"><div class="head"><span class="team-name">{(nameA != '' ? nameA : t("arena.team.a"))}</span><span part="ArenaScoreboard.score" class="team-score" data-team="a" hidden={scoreA == ''} aria-description={t("arena.score.teamScore")}>{scoreA}</span></div></th>
              <th class="num" scope="col" aria-label={t("arena.score.killsName")} hidden={columns != 'kda'}>{t("arena.score.kills")}</th>
              <th class="num" scope="col" aria-label={t("arena.score.deathsName")} hidden={columns != 'kda'}>{t("arena.score.deaths")}</th>
              <th class="num" scope="col" aria-label={t("arena.score.assistsName")} hidden={columns != 'kda'}>{t("arena.score.assists")}</th>
              <th class="total" scope="col">{t("arena.score.score")}</th>
            </tr>
          </thead>
          <tbody>
            {#each rows as r (r.id)}
              {#if r.team == 'a'}
                <tr part="ArenaScoreboard.row" aria-current={r.local} data-local={r.local} data-disconnected={r.disconnected}>
                  <th class="who-cell" scope="row"><div class="who"><span part="ArenaScoreboard.name" class="name">{r.name}</span><span part="ArenaScoreboard.badge" class="you" data-tag="you" hidden={!r.local}>{t("arena.you")}</span><span part="ArenaScoreboard.badge" class="off" data-tag="disconnected" hidden={!r.disconnected}>{t("arena.score.disconnected")}</span></div></th>
                  <td part="ArenaScoreboard.cell" class="num" hidden={columns != 'kda'}>{formatNumber(r.kills)}</td>
                  <td part="ArenaScoreboard.cell" class="num" hidden={columns != 'kda'}>{formatNumber(r.deaths)}</td>
                  <td part="ArenaScoreboard.cell" class="num" hidden={columns != 'kda'}>{formatNumber(r.assists)}</td>
                  <td part="ArenaScoreboard.cell" class="total">{formatNumber(r.score)}</td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
        <table part="ArenaScoreboard.team" class="team" aria-label={(nameB != '' ? nameB : t("arena.team.b"))}>
          <thead>
            <tr part="ArenaScoreboard.heading" class="heading" data-team="b">
              <th class="team-head" scope="col"><div class="head"><span class="team-name">{(nameB != '' ? nameB : t("arena.team.b"))}</span><span part="ArenaScoreboard.score" class="team-score" data-team="b" hidden={scoreB == ''} aria-description={t("arena.score.teamScore")}>{scoreB}</span></div></th>
              <th class="num" scope="col" aria-label={t("arena.score.killsName")} hidden={columns != 'kda'}>{t("arena.score.kills")}</th>
              <th class="num" scope="col" aria-label={t("arena.score.deathsName")} hidden={columns != 'kda'}>{t("arena.score.deaths")}</th>
              <th class="num" scope="col" aria-label={t("arena.score.assistsName")} hidden={columns != 'kda'}>{t("arena.score.assists")}</th>
              <th class="total" scope="col">{t("arena.score.score")}</th>
            </tr>
          </thead>
          <tbody>
            {#each rows as r (r.id)}
              {#if r.team == 'b'}
                <tr part="ArenaScoreboard.row" aria-current={r.local} data-local={r.local} data-disconnected={r.disconnected}>
                  <th class="who-cell" scope="row"><div class="who"><span part="ArenaScoreboard.name" class="name">{r.name}</span><span part="ArenaScoreboard.badge" class="you" data-tag="you" hidden={!r.local}>{t("arena.you")}</span><span part="ArenaScoreboard.badge" class="off" data-tag="disconnected" hidden={!r.disconnected}>{t("arena.score.disconnected")}</span></div></th>
                  <td part="ArenaScoreboard.cell" class="num" hidden={columns != 'kda'}>{formatNumber(r.kills)}</td>
                  <td part="ArenaScoreboard.cell" class="num" hidden={columns != 'kda'}>{formatNumber(r.deaths)}</td>
                  <td part="ArenaScoreboard.cell" class="num" hidden={columns != 'kda'}>{formatNumber(r.assists)}</td>
                  <td part="ArenaScoreboard.cell" class="total">{formatNumber(r.score)}</td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
      </div>
    </scroll>
  {:else}
    <p class="empty-text" role="status">{t("arena.score.empty")}</p>
  {/if}
</section>

<style>
[part="ArenaScoreboard.frame"][hidden], [part="ArenaScoreboard.frame"] [hidden] { display: none; }
[part="ArenaScoreboard.frame"] { display: flex; flex-direction: column; width: stretch; min-width: 0; max-height: stretch; gap: var(--space-sm); container-type: inline-size; }
[part="ArenaScoreboard.band"] { display: flex; width: stretch; align-items: center; flex: none; }
[part="ArenaScoreboard.title"] { margin: 0; min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.list { width: stretch; flex: 1; min-height: 0; }
.teams { display: flex; flex-direction: column; width: stretch; gap: var(--space-md); }
[part="ArenaScoreboard.team"] { width: stretch; }
[part="ArenaScoreboard.team"] :is(th, td) { padding: var(--space-xxs) var(--space-sm); font-weight: inherit; }
[part="ArenaScoreboard.row"] { min-height: var(--size-md); }
/* The name column takes what the figures leave; the figures size to their widest value. */
.team-head, .who-cell { width: 100%; text-align: start; }
.head, .who { display: flex; align-items: center; gap: var(--space-xxs) var(--space-sm); min-width: 0; }
/* A long team name wraps beside its score; a player's badges wrap under the name. */
.who { flex-wrap: wrap; }
.team-name, .name { min-width: 0; text-align: start; overflow-wrap: anywhere; }
.team-score { flex: none; min-width: var(--size-md); text-align: center; white-space: nowrap; font-variant-numeric: tabular-nums; }
.you, .off { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
.num, .total { text-align: end; white-space: nowrap; font-variant-numeric: tabular-nums; }
.num { min-width: var(--size-md); }
.empty-text { margin: 0; padding: var(--space-lg); text-align: center; }
@container (width < 600px) {
  [part="ArenaScoreboard.team"] :is(th, td) { padding: var(--space-xxs) var(--space-xs); }
  /* A narrow board cannot hold a long name beside four figures at large
     text: the name takes its own line and the figures share the line under
     it in even columns, the same in every row, so they still line up. */
  [part="ArenaScoreboard.team"], [part="ArenaScoreboard.team"] :is(thead, tbody) { display: block; }
  [part="ArenaScoreboard.team"] tr { display: flex; flex-wrap: wrap; align-items: center; width: stretch; }
  .team-head, .who-cell { flex: 1 1 100%; width: auto; min-width: 0; max-width: stretch; }
  .num { flex: 1 1 0; min-width: 0; }
  .total { flex: 2 1 0; min-width: 0; }
}
</style>
