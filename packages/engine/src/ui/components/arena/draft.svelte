<meta name="component" content="ArenaDraft">
<meta name="strings" content="./arena.en.ogt">
<meta name="parts" content="ArenaDraft.frame ArenaDraft.band ArenaDraft.title ArenaDraft.team ArenaDraft.heading ArenaDraft.slot ArenaDraft.portrait ArenaDraft.name ArenaDraft.badge">
<meta name="prop:title" content="string" data-doc="The draft's heading; empty takes the kit's (Draft).">
<meta name="prop:phase" content="string" data-doc="The phase the game is in: ban, pick or done (each named in words in the band).">
<meta name="prop:active" content="string" data-doc="The team choosing now: a, b, or empty for neither (its column is marked current and named in the band).">
<meta name="prop:nameA" content="string" data-doc="The first team's name; empty takes the kit's (Blue team).">
<meta name="prop:nameB" content="string" data-doc="The second team's name; empty takes the kit's (Red team).">
<meta name="prop:slots" content="list" data-doc="Both teams' ban and pick slots in the game's order, keyed rows of { id, team, kind, name, portrait, icon, player, active }: team a or b; kind ban or pick; name the character placed (empty: the slot is open); portrait a texture (0 shows the icon); player who the pick belongs to (empty hides); active the slot being filled now (Banning, Picking).">
<meta name="prop:timeLeft" content="number" data-doc="Seconds left in the turn as the game counts them (a HudTimer); negative hides it.">
<meta name="prop:timeTotal" content="number" data-doc="The turn's whole time, for the timer's ring (0: no ring).">
<meta name="prop:canConfirm" content="bool" data-doc="Lock in is offered (false: disabled; the game decides when the local player may lock in).">
<meta name="action:details" content="Lock in">
<meta name="fixture:default" content='{"title":"","phase":"pick","active":"a","nameA":"","nameB":"","timeLeft":21,"timeTotal":30,"canConfirm":true,"slots":[{"id":"ab1","team":"a","kind":"ban","name":"Oarsman","portrait":0,"icon":"skull","player":"","active":false},{"id":"ab2","team":"a","kind":"ban","name":"Lurker","portrait":0,"icon":"drop","player":"","active":false},{"id":"ap1","team":"a","kind":"pick","name":"Brenna","portrait":0,"icon":"shield","player":"Kai","active":false},{"id":"ap2","team":"a","kind":"pick","name":"","portrait":0,"icon":"","player":"You","active":true},{"id":"ap3","team":"a","kind":"pick","name":"","portrait":0,"icon":"","player":"Ana","active":false},{"id":"bb1","team":"b","kind":"ban","name":"Sela","portrait":0,"icon":"heart","player":"","active":false},{"id":"bb2","team":"b","kind":"ban","name":"Idris","portrait":0,"icon":"bolt","player":"","active":false},{"id":"bp1","team":"b","kind":"pick","name":"Vex","portrait":0,"icon":"sword","player":"Rook","active":false},{"id":"bp2","team":"b","kind":"pick","name":"Maren","portrait":0,"icon":"leaf","player":"Wren","active":false},{"id":"bp3","team":"b","kind":"pick","name":"","portrait":0,"icon":"","player":"Quill","active":false}]}'>
<meta name="fixture:ban" content='{"title":"","phase":"ban","active":"b","nameA":"","nameB":"","timeLeft":6,"timeTotal":20,"canConfirm":false,"slots":[{"id":"ab1","team":"a","kind":"ban","name":"Oarsman","portrait":0,"icon":"skull","player":"","active":false},{"id":"ab2","team":"a","kind":"ban","name":"","portrait":0,"icon":"","player":"","active":false},{"id":"ap1","team":"a","kind":"pick","name":"","portrait":0,"icon":"","player":"Kai","active":false},{"id":"ap2","team":"a","kind":"pick","name":"","portrait":0,"icon":"","player":"You","active":false},{"id":"bb1","team":"b","kind":"ban","name":"","portrait":0,"icon":"","player":"","active":true},{"id":"bb2","team":"b","kind":"ban","name":"","portrait":0,"icon":"","player":"","active":false},{"id":"bp1","team":"b","kind":"pick","name":"","portrait":0,"icon":"","player":"Rook","active":false},{"id":"bp2","team":"b","kind":"pick","name":"","portrait":0,"icon":"","player":"Wren","active":false}]}'>
<meta name="fixture:empty" content='{"title":"","phase":"ban","active":"","nameA":"","nameB":"","timeLeft":-1,"timeTotal":0,"canConfirm":false,"slots":[]}'>
<meta name="fixture:done" content='{"title":"","phase":"done","active":"","nameA":"Iron Wolves","nameB":"Ember Court","timeLeft":-1,"timeTotal":0,"canConfirm":false,"slots":[{"id":"ab1","team":"a","kind":"ban","name":"Oarsman","portrait":0,"icon":"skull","player":"","active":false},{"id":"ap1","team":"a","kind":"pick","name":"Brenna","portrait":0,"icon":"shield","player":"Kai","active":false},{"id":"ap2","team":"a","kind":"pick","name":"Vex","portrait":0,"icon":"sword","player":"You","active":false},{"id":"bb1","team":"b","kind":"ban","name":"Sela","portrait":0,"icon":"heart","player":"","active":false},{"id":"bp1","team":"b","kind":"pick","name":"Idris","portrait":0,"icon":"bolt","player":"Rook","active":false},{"id":"bp2","team":"b","kind":"pick","name":"Maren","portrait":0,"icon":"leaf","player":"Wren","active":false}]}'>
<meta name="fixture:overflow" content='{"title":"Championship Series · Grand Finals · Deciding match draft","phase":"pick","active":"b","nameA":"The Unbroken Wardens of the Ninefold Gate","nameB":"Ember Court Invitational Reserve","timeLeft":3,"timeTotal":30,"canConfirm":false,"slots":[{"id":"ab1","team":"a","kind":"ban","name":"Ysolde the Unbroken, Warden of the Ninefold Gate","portrait":0,"icon":"crown","player":"","active":false},{"id":"ab2","team":"a","kind":"ban","name":"Lurker","portrait":0,"icon":"drop","player":"","active":false},{"id":"ab3","team":"a","kind":"ban","name":"Hollow","portrait":0,"icon":"gem","player":"","active":false},{"id":"ap1","team":"a","kind":"pick","name":"Brenna","portrait":0,"icon":"shield","player":"Maximilian Vandenberghe-Okonkwo","active":false},{"id":"ap2","team":"a","kind":"pick","name":"Vex","portrait":0,"icon":"sword","player":"You","active":false},{"id":"ap3","team":"a","kind":"pick","name":"Tamsin","portrait":0,"icon":"music","player":"Ana","active":false},{"id":"ap4","team":"a","kind":"pick","name":"Gorrum","portrait":0,"icon":"bomb","player":"Kai","active":false},{"id":"ap5","team":"a","kind":"pick","name":"Quill","portrait":0,"icon":"star","player":"Jo","active":false},{"id":"bb1","team":"b","kind":"ban","name":"Sela","portrait":0,"icon":"heart","player":"","active":false},{"id":"bb2","team":"b","kind":"ban","name":"Idris","portrait":0,"icon":"bolt","player":"","active":false},{"id":"bb3","team":"b","kind":"ban","name":"Ashen Pyre","portrait":0,"icon":"fire","player":"","active":false},{"id":"bp1","team":"b","kind":"pick","name":"Maren","portrait":0,"icon":"leaf","player":"Rook","active":false},{"id":"bp2","team":"b","kind":"pick","name":"Kestrel","portrait":0,"icon":"flag","player":"Wren","active":false},{"id":"bp3","team":"b","kind":"pick","name":"Wren","portrait":0,"icon":"apple","player":"Abernathy-Whitcombe","active":false},{"id":"bp4","team":"b","kind":"pick","name":"","portrait":0,"icon":"","player":"Moreau","active":true},{"id":"bp5","team":"b","kind":"pick","name":"","portrait":0,"icon":"","player":"Laurent","active":false}]}'>

<!--
  A ban and pick draft between two teams: the band names the phase (Ban
  phase, Pick phase, Draft complete) and the team to choose, with the
  turn's countdown as the game counts it (HudTimer); a column per team
  holds its bans (struck, Banned) and its picks (the character and its
  player, or Open), the slot being filled marked Banning or Picking. Lock
  in (confirm; also the details action, X or pad Y) reports once and is disabled when the
  game refuses. The draft decides no turn, legality or timing.
-->
<section id="ArenaDraft" part="ArenaDraft.frame" aria-label={title != '' ? title : t("arena.draft.title")}>
  <header part="ArenaDraft.band" class="band">
    <div class="head">
      <h2 part="ArenaDraft.title">{title != '' ? title : t("arena.draft.title")}</h2>
      <span part="ArenaDraft.badge" class="phase" data-phase={phase} hidden={!(phase == 'ban' || phase == 'pick' || phase == 'done')}>{phase == 'ban' ? t("arena.draft.ban") : (phase == 'pick' ? t("arena.draft.pick") : (phase == 'done' ? t("arena.draft.done") : ''))}</span>
      <span class="turn" role="status" hidden={!(active == 'a' || active == 'b')}>{t("arena.draft.turn", { team: active == 'a' ? (nameA != '' ? nameA : t("arena.team.a")) : (nameB != '' ? nameB : t("arena.team.b")) })}</span>
    </div>
    <div class="clock" hidden={timeLeft < 0}><HudTimer seconds={timeLeft < 0 ? 0 : timeLeft} total={timeTotal} countdown={true} warnAt={10} paused={false} label={t("arena.draft.timeLeft")} /></div>
  </header>
  {#if slots.length}
    <div class="teams">
      <div part="ArenaDraft.team" class="team" aria-current={active == 'a'} role="group" aria-label={(nameA != '' ? nameA : t("arena.team.a"))}>
        <h3 part="ArenaDraft.heading" class="team-name" data-team="a">{(nameA != '' ? nameA : t("arena.team.a"))}</h3>
        <span class="section">{t("arena.draft.bans")}</span>
        <div class="bans" role="list" aria-label={t("arena.draft.bans")}>
          {#each slots as s (s.id)}
            {#if s.team == 'a' && s.kind == 'ban'}
              <div part="ArenaDraft.slot" class="ban" role="listitem" aria-current={s.active} data-kind="ban" data-state={s.active ? 'active' : (s.name != '' ? 'banned' : 'empty')}>
                <div part="ArenaDraft.portrait" class="portrait small" data-state={s.name != '' ? 'banned' : 'empty'}>
                  <img class="art" src={s.portrait} alt="" hidden={!s.portrait}>
                  <og-icon class="glyph" icon={s.icon} hidden={!!s.portrait || s.icon == ''}></og-icon>
                  <og-icon class="strike" icon="close" hidden={s.name == ''}></og-icon>
                </div>
                <span part="ArenaDraft.badge" class="tag" data-tag={s.active ? 'banning' : (s.name != '' ? 'banned' : 'open')}>{s.active ? t("arena.draft.banning") : (s.name != '' ? t("arena.draft.banned") : t("arena.draft.open"))}</span>
                <span part="ArenaDraft.name" class="ban-name" hidden={s.name == ''}>{s.name}</span>
              </div>
            {/if}
          {/each}
        </div>
        <span class="section">{t("arena.draft.picks")}</span>
        <div class="picks" role="list" aria-label={t("arena.draft.picks")}>
          {#each slots as s (s.id)}
            {#if s.team == 'a' && s.kind == 'pick'}
              <div part="ArenaDraft.slot" class="pick" role="listitem" aria-current={s.active} data-kind="pick" data-state={s.active ? 'active' : (s.name != '' ? 'filled' : 'empty')}>
                <div part="ArenaDraft.portrait" class="portrait" data-state={s.name != '' ? 'filled' : 'empty'}>
                  <img class="art" src={s.portrait} alt="" hidden={!s.portrait}>
                  <og-icon class="glyph" icon={s.icon} hidden={!!s.portrait || s.icon == ''}></og-icon>
                </div>
                <div class="who">
                  <span part="ArenaDraft.name" class="pick-name">{s.name != '' ? s.name : t("arena.draft.open")}</span>
                  <div class="line">
                    <span class="player" hidden={s.player == ''}>{s.player}</span>
                    <span part="ArenaDraft.badge" class="tag" data-tag="picking" hidden={!s.active}>{t("arena.draft.picking")}</span>
                  </div>
                </div>
              </div>
            {/if}
          {/each}
        </div>
      </div>
      <div part="ArenaDraft.team" class="team" aria-current={active == 'b'} role="group" aria-label={(nameB != '' ? nameB : t("arena.team.b"))}>
        <h3 part="ArenaDraft.heading" class="team-name" data-team="b">{(nameB != '' ? nameB : t("arena.team.b"))}</h3>
        <span class="section">{t("arena.draft.bans")}</span>
        <div class="bans" role="list" aria-label={t("arena.draft.bans")}>
          {#each slots as s (s.id)}
            {#if s.team == 'b' && s.kind == 'ban'}
              <div part="ArenaDraft.slot" class="ban" role="listitem" aria-current={s.active} data-kind="ban" data-state={s.active ? 'active' : (s.name != '' ? 'banned' : 'empty')}>
                <div part="ArenaDraft.portrait" class="portrait small" data-state={s.name != '' ? 'banned' : 'empty'}>
                  <img class="art" src={s.portrait} alt="" hidden={!s.portrait}>
                  <og-icon class="glyph" icon={s.icon} hidden={!!s.portrait || s.icon == ''}></og-icon>
                  <og-icon class="strike" icon="close" hidden={s.name == ''}></og-icon>
                </div>
                <span part="ArenaDraft.badge" class="tag" data-tag={s.active ? 'banning' : (s.name != '' ? 'banned' : 'open')}>{s.active ? t("arena.draft.banning") : (s.name != '' ? t("arena.draft.banned") : t("arena.draft.open"))}</span>
                <span part="ArenaDraft.name" class="ban-name" hidden={s.name == ''}>{s.name}</span>
              </div>
            {/if}
          {/each}
        </div>
        <span class="section">{t("arena.draft.picks")}</span>
        <div class="picks" role="list" aria-label={t("arena.draft.picks")}>
          {#each slots as s (s.id)}
            {#if s.team == 'b' && s.kind == 'pick'}
              <div part="ArenaDraft.slot" class="pick" role="listitem" aria-current={s.active} data-kind="pick" data-state={s.active ? 'active' : (s.name != '' ? 'filled' : 'empty')}>
                <div part="ArenaDraft.portrait" class="portrait" data-state={s.name != '' ? 'filled' : 'empty'}>
                  <img class="art" src={s.portrait} alt="" hidden={!s.portrait}>
                  <og-icon class="glyph" icon={s.icon} hidden={!!s.portrait || s.icon == ''}></og-icon>
                </div>
                <div class="who">
                  <span part="ArenaDraft.name" class="pick-name">{s.name != '' ? s.name : t("arena.draft.open")}</span>
                  <div class="line">
                    <span class="player" hidden={s.player == ''}>{s.player}</span>
                    <span part="ArenaDraft.badge" class="tag" data-tag="picking" hidden={!s.active}>{t("arena.draft.picking")}</span>
                  </div>
                </div>
              </div>
            {/if}
          {/each}
        </div>
      </div>
    </div>
  {:else}
    <p class="empty-text" role="status">{t("arena.draft.empty")}</p>
  {/if}
  <footer class="actions" hidden={phase == 'done'}>
    <button id="Confirm" variant="primary" disabled={!canConfirm || phase == 'done'} onclick={confirm} ondetails={confirm}><kbd class="key" data-action="details"></kbd><span>{t("arena.draft.confirm")}</span></button>
  </footer>
</section>

<style>
[part="ArenaDraft.frame"][hidden], [part="ArenaDraft.frame"] [hidden] { display: none; }
[part="ArenaDraft.frame"] { display: flex; flex-direction: column; width: stretch; min-width: 0; max-height: stretch; gap: var(--space-sm); container-type: inline-size; }
[part="ArenaDraft.band"] { display: flex; width: stretch; align-items: center; justify-content: space-between; gap: var(--space-md); flex: none; }
.head { display: flex; flex: 1; min-width: 0; align-items: center; flex-wrap: wrap; gap: var(--space-xs) var(--space-md); }
[part="ArenaDraft.title"] { margin: 0; min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.phase, .tag { flex: none; padding: 0 var(--space-xs); white-space: nowrap; }
.turn { min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.clock { flex: none; }
.teams { display: flex; width: stretch; flex: 1; min-height: 0; gap: var(--space-md); overflow-y: auto; }
[part="ArenaDraft.team"] { display: flex; flex-direction: column; flex: 1 1 0; min-width: 0; gap: var(--space-xs); padding: var(--space-sm); }
[part="ArenaDraft.heading"] { margin: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.section { white-space: nowrap; }
.bans { display: flex; flex-wrap: wrap; gap: var(--space-xs); }
.picks { display: flex; flex-direction: column; gap: var(--space-xs); }
/* A ban is as wide as its words (Banned, the name), at any text size: the word is never cut, a long name wraps. */
[part="ArenaDraft.slot"].ban { display: flex; flex-direction: column; flex: 0 1 auto; align-items: center; gap: var(--space-xxs); min-width: var(--size-xl); max-width: stretch; }
[part="ArenaDraft.slot"].pick { display: flex; width: stretch; align-items: center; gap: var(--space-sm); padding: var(--space-xxs) var(--space-xs); min-height: var(--size-touch); }
[part="ArenaDraft.portrait"] { position: relative; display: flex; flex: none; width: var(--size-lg); height: var(--size-lg); align-items: center; justify-content: center; overflow: hidden; }
[part="ArenaDraft.portrait"].small { width: var(--size-md); height: var(--size-md); }
.art { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.glyph { width: var(--size-sm); height: var(--size-sm); }
.small .glyph { width: var(--size-xs); height: var(--size-xs); }
.strike { position: absolute; left: 0; top: 0; width: stretch; height: stretch; }
.ban-name { max-width: stretch; text-align: center; overflow-wrap: anywhere; }
.who { display: flex; flex-direction: column; flex: 1; min-width: 0; }
.pick-name, .player { min-width: 0; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.line { display: flex; flex-wrap: wrap; min-width: 0; align-items: center; gap: var(--space-xxs) var(--space-xs); }
.line .tag { flex: none; }
.empty-text { margin: 0; padding: var(--space-lg); text-align: center; }
.actions { display: flex; width: stretch; flex: none; justify-content: flex-end; padding-bottom: env(safe-area-inset-bottom); }
.actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); gap: var(--space-xs); }
@media (pointer: coarse) { .key { display: none; } }
/* The draft sits in a panel: its own width, not the screen's, tightens it. */
@container (width < 600px) {
  [part="ArenaDraft.band"] { flex-wrap: wrap; gap: var(--space-xs); }
  .teams { gap: var(--space-xs); }
  [part="ArenaDraft.team"] { padding: var(--space-xs); }
  [part="ArenaDraft.slot"].pick { gap: var(--space-xs); padding: var(--space-xxs); }
  [part="ArenaDraft.portrait"] { width: var(--size-md); height: var(--size-md); }
  .actions button { flex: 1; }
}
</style>
