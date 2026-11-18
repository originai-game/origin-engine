<meta name="component" content="RacingStandings">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingStandings.frame RacingStandings.row RacingStandings.place RacingStandings.car RacingStandings.name RacingStandings.gap RacingStandings.badge">
<meta name="prop:title" content="string" data-doc="The tower's name for a reader; empty takes the kit's (Standings). Not drawn.">
<meta name="prop:rows" content="list" data-doc="The running order as the game ranks it, keyed rows of { id, position, name, gap, color, player, state }: position the place shown in its box; gap the interval as the game words it (+1.204, +1 lap; the leader's empty gap reads Leader); color the car's or team's colour as a CSS colour, on a stripe beside the name (empty: a neutral stripe); player marks the local player's row (lit in the HUD's accent, aria-current); state racing, pitting, retired or finished, each a word on a tag (Pit, DNF, Fin), a retired row dimmed.">
<meta name="fixture:default" content='{"title":"","rows":[{"id":"okafor","position":1,"name":"M. Okafor","gap":"","color":"#ffd24a","player":false,"state":"racing"},{"id":"lindqvist","position":2,"name":"S. Lindqvist","gap":"+1.204","color":"#4f7bff","player":false,"state":"racing"},{"id":"you","position":3,"name":"You","gap":"+1.861","color":"#ff4d5e","player":true,"state":"racing"},{"id":"castellanos","position":4,"name":"R. Castellanos","gap":"+0.337","color":"#55e39a","player":false,"state":"racing"},{"id":"nakamura","position":5,"name":"T. Nakamura","gap":"+2.950","color":"#c98bff","player":false,"state":"pitting"}]}'>
<meta name="fixture:leading" content='{"title":"","rows":[{"id":"you","position":1,"name":"You","gap":"","color":"#ff4d5e","player":true,"state":"racing"},{"id":"okafor","position":2,"name":"M. Okafor","gap":"+0.884","color":"#ffd24a","player":false,"state":"racing"},{"id":"lindqvist","position":3,"name":"S. Lindqvist","gap":"+2.310","color":"#4f7bff","player":false,"state":"racing"}]}'>
<meta name="fixture:last" content='{"title":"","rows":[{"id":"okafor","position":1,"name":"M. Okafor","gap":"","color":"#ffd24a","player":false,"state":"racing"},{"id":"lindqvist","position":2,"name":"S. Lindqvist","gap":"+1.204","color":"#4f7bff","player":false,"state":"racing"},{"id":"castellanos","position":3,"name":"R. Castellanos","gap":"+0.337","color":"#55e39a","player":false,"state":"racing"},{"id":"nakamura","position":4,"name":"T. Nakamura","gap":"+2.950","color":"#c98bff","player":false,"state":"racing"},{"id":"brandt","position":5,"name":"L. Brandt","gap":"+3.107","color":"","player":false,"state":"racing"},{"id":"sato","position":6,"name":"K. Sato","gap":"+4.020","color":"#38e8ff","player":false,"state":"pitting"},{"id":"you","position":7,"name":"You","gap":"+6.512","color":"#ff4d5e","player":true,"state":"racing"},{"id":"marlow","position":8,"name":"J. Marlow","gap":"","color":"#ff9a48","player":false,"state":"retired"}]}'>
<meta name="fixture:empty" content='{"title":"","rows":[]}'>
<meta name="fixture:single" content='{"title":"Time trial","rows":[{"id":"you","position":1,"name":"You","gap":"","color":"","player":true,"state":"racing"}]}'>
<meta name="fixture:finished" content='{"title":"","rows":[{"id":"okafor","position":1,"name":"M. Okafor","gap":"","color":"#ffd24a","player":false,"state":"finished"},{"id":"you","position":2,"name":"You","gap":"+0.207","color":"#ff4d5e","player":true,"state":"finished"},{"id":"lindqvist","position":3,"name":"S. Lindqvist","gap":"+2.940","color":"#4f7bff","player":false,"state":"racing"}]}'>
<meta name="fixture:overflow" content='{"title":"","rows":[{"id":"r1","position":1,"name":"Maximilian Vandenberghe-Okonkwo","gap":"","color":"#ffd24a","player":false,"state":"racing"},{"id":"r2","position":2,"name":"Okafor","gap":"+0.884","color":"#4f7bff","player":false,"state":"racing"},{"id":"r3","position":3,"name":"Lindqvist","gap":"+1.020","color":"","player":false,"state":"racing"},{"id":"r4","position":4,"name":"Sato","gap":"+2.310","color":"","player":false,"state":"racing"},{"id":"r5","position":5,"name":"Brandt","gap":"+3.007","color":"","player":false,"state":"pitting"},{"id":"r6","position":6,"name":"Duarte","gap":"+4.551","color":"","player":false,"state":"racing"},{"id":"r7","position":7,"name":"Nakamura","gap":"+5.902","color":"","player":false,"state":"racing"},{"id":"r8","position":8,"name":"Haddad","gap":"+7.118","color":"","player":false,"state":"racing"},{"id":"r9","position":9,"name":"Kowalczyk","gap":"+8.640","color":"","player":false,"state":"racing"},{"id":"r10","position":10,"name":"Moreau","gap":"+9.215","color":"","player":false,"state":"racing"},{"id":"r11","position":11,"name":"Rhea Vance-Abernathy of the Coastal Racing Club","gap":"+10.774","color":"#ff4d5e","player":true,"state":"racing"},{"id":"r12","position":12,"name":"Ferreira","gap":"+12.330","color":"","player":false,"state":"racing"},{"id":"r13","position":13,"name":"Johansson","gap":"+15.004","color":"","player":false,"state":"racing"},{"id":"r14","position":14,"name":"Abernathy-Whitcombe","gap":"+1 lap","color":"","player":false,"state":"racing"},{"id":"r15","position":15,"name":"Ivanova","gap":"+1 lap","color":"","player":false,"state":"racing"},{"id":"r16","position":16,"name":"Quinn","gap":"+2 laps","color":"","player":false,"state":"racing"},{"id":"r17","position":17,"name":"Petrov","gap":"","color":"","player":false,"state":"retired"},{"id":"r18","position":18,"name":"Laurent","gap":"","color":"","player":false,"state":"retired"}]}'>

<!--
  The position tower at the left edge: one leaning glass row per car in
  the order the game ranks them, the place in a box, the car's colour on a
  stripe, the name and the gap (Leader for the leader). The player's row
  is lit in the HUD's accent; pitting, retired and finished cars carry a
  word on a tag. On a phone it collapses to the leader and the rows
  either side of the player. Read only.
-->
<section id="RacingStandings" part="RacingStandings.frame" class="hud bare" data-layout="stack" aria-label={title != '' ? title : t("racing.standings.name")}>
  {#if rows.length}
    <div class="list" data-layout="scroll">
      <div class="rows" data-layout="stack" role="list" aria-label={title != '' ? title : t("racing.standings.name")}>
        {#each rows as r (r.id)}
          <div part="RacingStandings.row" class="row" role="listitem" aria-current={r.player} aria-label={r.name} data-player={r.player} data-state={r.state} style:--hud-car={r.color != '' ? r.color : null}>
            <span part="RacingStandings.place" class="place">{formatNumber(r.position)}</span>
            <span part="RacingStandings.car" class="car"></span>
            <span part="RacingStandings.name" class="name">{r.name}</span>
            {#if r.state != 'racing'}<span part="RacingStandings.badge" class="state" data-state={r.state} aria-label={r.state == 'pitting' ? t("racing.standings.pittingName") : (r.state == 'retired' ? t("racing.standings.retiredName") : t("racing.standings.finishedName"))}>{r.state == 'pitting' ? t("racing.standings.pitting") : (r.state == 'retired' ? t("racing.standings.retired") : t("racing.standings.finished"))}</span>{/if}
            {#if r.gap != '' || (r.position == 1 && r.state != 'retired')}<span part="RacingStandings.gap" class="gap" data-leader={r.gap == ''}>{r.gap != '' ? r.gap : t("racing.standings.leader")}</span>{/if}
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <p class="empty" part="RacingStandings.badge" role="status">{t("racing.standings.empty")}</p>
  {/if}
</section>

<style>
:host { inline-size: calc(250 * var(--font-step-px)); max-inline-size: stretch; max-block-size: stretch; color: var(--hud-ink); }
.list { inline-size: stretch; min-block-size: 0; }
.rows { inline-size: stretch; row-gap: calc(4 * var(--font-step-px)); }
/* A row: the place's box, the car's stripe, the name and the gap, on
   dark glass fading to the right, two corners cut. */
.row { display: flex; align-items: center; column-gap: calc(8 * var(--font-step-px)); inline-size: stretch; min-inline-size: 0; padding-inline-end: calc(12 * var(--font-step-px)); background: var(--hud-feed-fill-start); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
.place { flex: none; inline-size: calc(30 * var(--font-step-px)); line-height: calc(26 * var(--font-step-px)); text-align: center; font-size: var(--hud-size-name); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; background: var(--hud-place-fill); }
.car { flex: none; inline-size: calc(5 * var(--font-step-px)); block-size: calc(20 * var(--font-step-px)); background: var(--hud-car); }
.name { flex: 1 1 auto; min-inline-size: 0; font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.gap { flex: none; font-size: var(--hud-size-name); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.gap[data-leader=true] { color: var(--hud-ink-dim); }
.state { flex: none; padding: 0 calc(5 * var(--font-step-px)); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
.state[data-state=pitting] { color: var(--hud-gold); }
.state[data-state=retired] { color: var(--hud-harm); }
.row[data-state=retired] { opacity: 0.6; }
/* The player's row, lit in the accent with ink to match. */
.row[data-player=true] { color: var(--hud-accent-ink); background: var(--hud-player-fill); }
.row[data-player=true] .place { background: var(--hud-place-player-fill); }
.row[data-player=true] .gap { color: var(--hud-accent-ink); }
.empty { margin: 0; padding: calc(4 * var(--font-step-px)) var(--space-sm); font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); background: var(--hud-feed-fill-start); white-space: nowrap; }
/* In a size container too short for the tower (a page that gives it the
   height left in its column, at a large text scale) it keeps the leader
   and the rows either side of the player, then the leader and the player
   alone, so no row is cut. */
@container (height < 11rem) {
  .row { display: none; }
  .row:first-child, .row[data-player=true], .row[data-player=true] + .row, .row:has(+ .row[data-player=true]) { display: flex; }
}
@container (height < 6.5rem) {
  .row[data-player=true] + .row, .row:has(+ .row[data-player=true]) { display: none; }
  .row:first-child { display: flex; }
}
/* On a phone the tower keeps the player's row and the rows either side of
   it, slimmer (the first three when no row is the player's). */
@media (width < 600px) {
  :host { inline-size: stretch; }
  .rows { row-gap: calc(2 * var(--font-step-px)); }
  .row { display: none; column-gap: calc(6 * var(--font-step-px)); padding-inline-end: calc(8 * var(--font-step-px)); }
  .row[data-player=true], .row[data-player=true] + .row, .row:has(+ .row[data-player=true]), .rows:not(:has(.row[data-player=true])) .row:nth-child(-n+3) { display: flex; }
  .place { inline-size: calc(24 * var(--font-step-px)); line-height: calc(20 * var(--font-step-px)); font-size: var(--hud-size-xs); }
  .car { block-size: calc(14 * var(--font-step-px)); }
  .name, .gap { font-size: var(--hud-size-xs); }
}
</style>
