<meta name="component" content="ArenaDraft">
<meta name="strings" content="./arena.en.ogt">
<meta name="parts" content="ArenaDraft.frame ArenaDraft.band ArenaDraft.title ArenaDraft.team ArenaDraft.heading ArenaDraft.slot ArenaDraft.portrait ArenaDraft.name ArenaDraft.badge ArenaDraft.plate ArenaDraft.picture ArenaDraft.caption ArenaDraft.headline ArenaDraft.row ArenaDraft.card">
<meta name="prop:title" content="string" data-doc="The draft's heading; empty takes the kit's (Draft).">
<meta name="prop:phase" content="string" data-doc="The phase the game is in: ban, pick or done (each named in words in the band).">
<meta name="prop:active" content="string" data-doc="The team choosing now: a, b, or empty for neither (its column is marked current and named in the band).">
<meta name="prop:nameA" content="string" data-doc="The first team's name; empty takes the kit's (Blue team).">
<meta name="prop:nameB" content="string" data-doc="The second team's name; empty takes the kit's (Red team).">
<meta name="prop:slots" content="list" data-doc="Both teams' ban and pick slots in the game's order, keyed rows of { id, team, kind, name, portrait, icon, player, active }: team a or b; kind ban or pick; name the character placed (empty: the slot is open); portrait a texture (0 shows the icon); player who the pick belongs to (empty hides); active the slot being filled now (Banning, Picking).">
<meta name="prop:timeLeft" content="number" data-default="-1" data-doc="Seconds left in the turn as the game counts them (a HudTimer); negative hides it.">
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
<section id="ArenaDraft" part="ArenaDraft.frame" data-layout="shell" data-gap="loose" aria-label={title != '' ? title : t("arena.draft.title")}>
  <header part="ArenaDraft.band" class="band" data-layout="cluster" data-gap="loose">
    <div class="head" data-layout="cluster" data-gap="loose" data-fill>
      <h2 part="ArenaDraft.title">{title != '' ? title : t("arena.draft.title")}</h2>
      {#if phase == 'ban' || phase == 'pick' || phase == 'done'}<span part="ArenaDraft.badge" class="phase" data-phase={phase}>{phase == 'ban' ? t("arena.draft.ban") : (phase == 'pick' ? t("arena.draft.pick") : (phase == 'done' ? t("arena.draft.done") : ''))}</span>{/if}
      {#if active == 'a' || active == 'b'}<span class="turn" part="ArenaDraft.caption" role="status">{t("arena.draft.turn", { team: active == 'a' ? (nameA != '' ? nameA : t("arena.team.a")) : (nameB != '' ? nameB : t("arena.team.b")) })}</span>{/if}
    </div>
    {#if timeLeft >= 0}<div class="clock"><HudTimer seconds={timeLeft < 0 ? 0 : timeLeft} total={timeTotal} countdown={true} warnAt={10} paused={false} label={t("arena.draft.timeLeft")} /></div>{/if}
  </header>
  {#if slots.length}
    <div class="teams" data-layout="row" data-gap="loose" data-fill>
      <div part="ArenaDraft.team ArenaDraft.plate ArenaDraft.card" class="team" data-team="a" data-layout="stack" data-gap="tight" data-fill aria-current={active == 'a'} role="group" aria-label={(nameA != '' ? nameA : t("arena.team.a"))}>
        <h3 part="ArenaDraft.heading" class="team-name" data-team="a">{(nameA != '' ? nameA : t("arena.team.a"))}</h3>
        <span class="section" part="ArenaDraft.caption">{t("arena.draft.bans")}</span>
        <div class="bans" data-layout="cluster" data-gap="tight" role="list" aria-label={t("arena.draft.bans")}>
          {#each slots as s (s.id)}
            {#if s.team == 'a' && s.kind == 'ban'}
              <div part="ArenaDraft.slot" class="ban" data-layout="stack" data-gap="hair" role="listitem" aria-current={s.active} data-kind="ban" data-state={s.active ? 'active' : (s.name != '' ? 'banned' : 'empty')}>
                <div part="ArenaDraft.portrait ArenaDraft.picture" class="portrait small" data-layout="ratio" data-state={s.name != '' ? 'banned' : 'empty'}>
                  <div class="well" data-layout="ratio">{#if s.portrait}<img class="art" src={s.portrait} alt="">{:else if s.icon != ''}<og-icon class="glyph" icon={s.icon}></og-icon>{/if}</div>
                  {#if s.name != ''}<og-icon class="strike" icon="close"></og-icon>{/if}
                </div>
                <span part="ArenaDraft.badge" class="tag" data-tag={s.active ? 'banning' : (s.name != '' ? 'banned' : 'open')}>{s.active ? t("arena.draft.banning") : (s.name != '' ? t("arena.draft.banned") : t("arena.draft.open"))}</span>
                {#if s.name != ''}<span part="ArenaDraft.name ArenaDraft.caption" class="ban-name">{s.name}</span>{/if}
              </div>
            {/if}
          {/each}
        </div>
        <span class="section" part="ArenaDraft.caption">{t("arena.draft.picks")}</span>
        <div class="picks" data-layout="stack" data-gap="tight" role="list" aria-label={t("arena.draft.picks")}>
          {#each slots as s (s.id)}
            {#if s.team == 'a' && s.kind == 'pick'}
              <div part="ArenaDraft.slot ArenaDraft.row" class="pick" data-layout="row" role="listitem" aria-current={s.active} data-kind="pick" data-state={s.active ? 'active' : (s.name != '' ? 'filled' : 'empty')}>
                <div part="ArenaDraft.portrait ArenaDraft.picture" class="portrait" data-layout="ratio" data-state={s.name != '' ? 'filled' : 'empty'}>
                  <div class="well" data-layout="ratio">{#if s.portrait}<img class="art" src={s.portrait} alt="">{:else if s.icon != ''}<og-icon class="glyph" icon={s.icon}></og-icon>{/if}</div>
                </div>
                <div class="who" data-layout="stack" data-gap="none" data-fill>
                  <span part="ArenaDraft.name ArenaDraft.headline" class="pick-name">{s.name != '' ? s.name : t("arena.draft.open")}</span>
                  <div class="line" data-layout="cluster" data-gap="tight">
                    {#if s.player != ''}<span class="player" part="ArenaDraft.caption">{s.player}</span>{/if}
                    {#if s.active}<span part="ArenaDraft.badge" class="tag" data-tag="picking">{t("arena.draft.picking")}</span>{/if}
                  </div>
                </div>
              </div>
            {/if}
          {/each}
        </div>
      </div>
      <div part="ArenaDraft.team ArenaDraft.plate ArenaDraft.card" class="team" data-team="b" data-layout="stack" data-gap="tight" data-fill aria-current={active == 'b'} role="group" aria-label={(nameB != '' ? nameB : t("arena.team.b"))}>
        <h3 part="ArenaDraft.heading" class="team-name" data-team="b">{(nameB != '' ? nameB : t("arena.team.b"))}</h3>
        <span class="section" part="ArenaDraft.caption">{t("arena.draft.bans")}</span>
        <div class="bans" data-layout="cluster" data-gap="tight" role="list" aria-label={t("arena.draft.bans")}>
          {#each slots as s (s.id)}
            {#if s.team == 'b' && s.kind == 'ban'}
              <div part="ArenaDraft.slot" class="ban" data-layout="stack" data-gap="hair" role="listitem" aria-current={s.active} data-kind="ban" data-state={s.active ? 'active' : (s.name != '' ? 'banned' : 'empty')}>
                <div part="ArenaDraft.portrait ArenaDraft.picture" class="portrait small" data-layout="ratio" data-state={s.name != '' ? 'banned' : 'empty'}>
                  <div class="well" data-layout="ratio">{#if s.portrait}<img class="art" src={s.portrait} alt="">{:else if s.icon != ''}<og-icon class="glyph" icon={s.icon}></og-icon>{/if}</div>
                  {#if s.name != ''}<og-icon class="strike" icon="close"></og-icon>{/if}
                </div>
                <span part="ArenaDraft.badge" class="tag" data-tag={s.active ? 'banning' : (s.name != '' ? 'banned' : 'open')}>{s.active ? t("arena.draft.banning") : (s.name != '' ? t("arena.draft.banned") : t("arena.draft.open"))}</span>
                {#if s.name != ''}<span part="ArenaDraft.name ArenaDraft.caption" class="ban-name">{s.name}</span>{/if}
              </div>
            {/if}
          {/each}
        </div>
        <span class="section" part="ArenaDraft.caption">{t("arena.draft.picks")}</span>
        <div class="picks" data-layout="stack" data-gap="tight" role="list" aria-label={t("arena.draft.picks")}>
          {#each slots as s (s.id)}
            {#if s.team == 'b' && s.kind == 'pick'}
              <div part="ArenaDraft.slot ArenaDraft.row" class="pick" data-layout="row" role="listitem" aria-current={s.active} data-kind="pick" data-state={s.active ? 'active' : (s.name != '' ? 'filled' : 'empty')}>
                <div part="ArenaDraft.portrait ArenaDraft.picture" class="portrait" data-layout="ratio" data-state={s.name != '' ? 'filled' : 'empty'}>
                  <div class="well" data-layout="ratio">{#if s.portrait}<img class="art" src={s.portrait} alt="">{:else if s.icon != ''}<og-icon class="glyph" icon={s.icon}></og-icon>{/if}</div>
                </div>
                <div class="who" data-layout="stack" data-gap="none" data-fill>
                  <span part="ArenaDraft.name ArenaDraft.headline" class="pick-name">{s.name != '' ? s.name : t("arena.draft.open")}</span>
                  <div class="line" data-layout="cluster" data-gap="tight">
                    {#if s.player != ''}<span class="player" part="ArenaDraft.caption">{s.player}</span>{/if}
                    {#if s.active}<span part="ArenaDraft.badge" class="tag" data-tag="picking">{t("arena.draft.picking")}</span>{/if}
                  </div>
                </div>
              </div>
            {/if}
          {/each}
        </div>
      </div>
    </div>
  {:else}
    <p class="empty-text" part="ArenaDraft.caption" role="status">{t("arena.draft.empty")}</p>
  {/if}
  {#if phase != 'done'}<footer class="actions" data-layout="row">
    <button id="Confirm" class="go" variant="primary" aria-label={t("arena.draft.confirm")} disabled={!canConfirm || phase == 'done'} onclick={confirm} ondetails={confirm}><kbd class="key" data-action="details"></kbd><span>{t("arena.draft.confirm")}</span></button>
  </footer>{/if}
</section>

<style>
/* The draft on a smoked cut card: a team's name in its colour over a rule
   of it, its bans as struck octagons, its picks as rows washed in the
   team's colour from their edge; the slot being filled is lit white. */
:host { inline-size: stretch; min-inline-size: 0; max-block-size: stretch; color: var(--hud-ink); }
[part="ArenaDraft.frame"] { box-sizing: border-box; padding: var(--space-lg); border: 0; border-radius: 0; color: var(--hud-ink); background: var(--party-card-fill); box-shadow: var(--party-card-rim); clip-path: polygon(var(--party-cut) 0, 100% 0, 100% calc(100% - var(--party-cut)), calc(100% - var(--party-cut)) 100%, 0 100%, 0 var(--party-cut)); }
[part="ArenaDraft.band"] { width: stretch; justify-content: space-between; flex: none; min-height: 0; padding: 0; border: 0; border-radius: 0; background: none; box-shadow: none; }
.head { min-width: 0; }
[part="ArenaDraft.title"] { margin: 0; min-inline-size: 0; font-size: var(--hud-size-md); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: none; white-space: normal; overflow-wrap: break-word; }
.phase, .tag { flex: none; padding: 0 var(--space-xs); border: 0; border-radius: 0; font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); text-shadow: none; clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); white-space: nowrap; }
.phase { color: var(--party-gold-ink); background: var(--hud-key-gold-fill); box-shadow: none; }
.tag[data-tag=banning], .tag[data-tag=picking] { color: var(--party-go-ink); background: var(--hud-accent); box-shadow: none; }
.tag[data-tag=open] { color: var(--hud-ink-dim); }
.turn { min-width: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-label-weight); color: var(--hud-ink); overflow-wrap: break-word; }
.clock { flex: none; }
.teams { width: stretch; min-height: 0; align-items: flex-start; overflow-y: auto; -og-scroll-fade: var(--space-xl); }
[part~="ArenaDraft.team"] { min-width: 0; padding: 0; border: 0; border-radius: 0; background: none; box-shadow: none; }
[part="ArenaDraft.heading"] { margin: 0; padding-block-end: var(--space-xs); font-size: var(--hud-size-readout); line-height: 1.15; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--result-team-a-ink); box-shadow: var(--result-team-a-rule); overflow-wrap: break-word; }
[part="ArenaDraft.heading"][data-team=b] { color: var(--result-team-b-ink); box-shadow: var(--result-team-b-rule); }
.section { font-size: var(--hud-label-size); line-height: 1.3; font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; }
/* A ban is as wide as its words (Banned, the name), at any text size: the word is never cut, a long name wraps. */
[part~="ArenaDraft.slot"].ban { flex: 0 1 auto; align-items: center; min-width: var(--size-xl); max-width: stretch; }
[part~="ArenaDraft.slot"].pick { padding: var(--space-xs) var(--space-sm); min-height: var(--size-touch); border-radius: 0; color: var(--hud-ink); background: var(--result-team-a-row); box-shadow: var(--result-team-a-edge); }
.team[data-team=b] [part~="ArenaDraft.slot"].pick { background: var(--result-team-b-row); box-shadow: var(--result-team-b-edge); }
[part~="ArenaDraft.slot"].pick[data-state=active] { background: var(--result-row-lit); color: var(--result-row-lit-ink); }
[part="ArenaDraft.portrait ArenaDraft.picture"] { position: relative; flex: none; box-sizing: border-box; padding: var(--stroke-thick); border: 0; border-radius: 0; background: var(--hud-edge); box-shadow: none; clip-path: var(--hud-octagon); overflow: hidden; inline-size: var(--party-portrait); block-size: var(--party-portrait); }
[part="ArenaDraft.portrait ArenaDraft.picture"].small { inline-size: var(--party-ban); block-size: var(--party-ban); }
[part="ArenaDraft.portrait ArenaDraft.picture"][data-state=empty] { padding: var(--stroke-hairline); background: var(--party-open-edge); }
.well { inline-size: stretch; block-size: stretch; clip-path: var(--hud-octagon); background: var(--party-open-fill); }
.art { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.glyph { width: var(--size-sm); height: var(--size-sm); color: var(--hud-ink); }
.small .glyph { width: var(--size-xs); height: var(--size-xs); }
[data-state=banned] .art { filter: var(--party-ban-art); }
[data-state=banned] .glyph { color: var(--hud-ink-dim); }
.strike { position: absolute; left: 0; top: 0; width: stretch; height: stretch; color: var(--hud-foe); }
.ban-name { max-width: stretch; text-align: center; font-size: var(--hud-size-xs); line-height: 1.2; color: var(--hud-ink-dim); overflow-wrap: anywhere; }
.who { min-width: 0; }
.pick-name { min-width: 0; text-align: start; font-size: var(--hud-size-name); line-height: 1.15; font-weight: var(--hud-figure-weight); color: inherit; overflow-wrap: break-word; }
[data-state=empty] .pick-name { color: var(--hud-ink-dim); }
.player { min-width: 0; text-align: start; font-size: var(--hud-size-xs); color: var(--hud-ink-dim); overflow-wrap: anywhere; }
[data-state=active] .player { color: var(--party-lit-dim); }
.line { min-width: 0; }
.empty-text { margin: 0; padding: var(--space-lg); text-align: center; font-size: var(--hud-size-sm); color: var(--hud-ink-dim); }
.actions { width: stretch; flex: none; justify-content: flex-end; padding-bottom: env(safe-area-inset-bottom); }
#Confirm { min-height: var(--size-touch); gap: var(--space-xs); padding: 0 var(--space-lg); font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); text-shadow: none; min-width: calc(var(--size-xxl) * 1.25);  }
#Confirm:focus-visible { box-shadow: var(--party-focus-rim); }
#Confirm:disabled { opacity: 0.45; }
#Confirm > span { color: inherit; font-weight: inherit; }
.key { color: var(--hud-key-ink); background: var(--hud-key-fill); box-shadow: var(--hud-key-shadow); border-radius: var(--hud-key-radius); }
@media (pointer: coarse) { .key { display: none; } }
/* The draft sits in a panel: its own width, not the screen's, tightens it. */
@container (width < 600px) {
  [part="ArenaDraft.frame"] { padding: var(--space-sm); }
  [part="ArenaDraft.band"] { --layout-gap-loose: var(--layout-gap-tight); }
  .teams { --layout-gap-loose: var(--layout-gap-tight); }
  [part~="ArenaDraft.slot"].pick { --layout-gap: var(--layout-gap-tight); padding: var(--space-xxs) var(--space-xs); }
  [part="ArenaDraft.portrait ArenaDraft.picture"] { inline-size: var(--party-portrait-compact); block-size: var(--party-portrait-compact); }
  #Confirm { flex: 1; }
}
@container (width < 520px) { .teams { flex-direction: column; align-items: stretch; } [part~="ArenaDraft.team"] { flex: none; } }
</style>
