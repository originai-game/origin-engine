<meta name="component" content="ArenaRoster">
<meta name="strings" content="./arena.en.ogt">
<meta name="parts" content="ArenaRoster.frame ArenaRoster.band ArenaRoster.title ArenaRoster.cell ArenaRoster.portrait ArenaRoster.name ArenaRoster.role ArenaRoster.badge ArenaRoster.mark">
<meta name="prop:title" content="string" data-doc="The screen's heading; empty takes the kit's (Choose your champion).">
<meta name="prop:characters" content="list" data-doc="The roster in the game's order, keyed rows of { id, name, portrait, icon, role, roleLabel, locked, takenBy }: portrait a texture (og.load.texture, 0 shows the icon instead); icon an icon name; role a key for the pack (tank, damage, support…) and roleLabel its word; locked the player cannot pick it yet; takenBy who already holds it (empty: free). Locked and taken cells are disabled and say why in words.">
<meta name="prop:selected" content="string" data-doc="The local player's character (aria-selected, named under the grid); empty is none.">
<meta name="prop:players" content="list" data-doc="Who is choosing, keyed rows of { id, slot, at, ready }: slot 1 to 4 (the mark P1…P4), at the character id the player is on (empty: none), ready the player has locked in (a check beside the mark).">
<meta name="fixture:default" content='{"title":"","selected":"vex","characters":[{"id":"brenna","name":"Brenna","portrait":0,"icon":"shield","role":"tank","roleLabel":"Vanguard","locked":false,"takenBy":""},{"id":"vex","name":"Vex","portrait":0,"icon":"sword","role":"damage","roleLabel":"Striker","locked":false,"takenBy":""},{"id":"sela","name":"Sela","portrait":0,"icon":"heart","role":"support","roleLabel":"Mender","locked":false,"takenBy":""},{"id":"idris","name":"Idris","portrait":0,"icon":"bolt","role":"damage","roleLabel":"Arcanist","locked":false,"takenBy":"Kai"},{"id":"maren","name":"Maren","portrait":0,"icon":"leaf","role":"support","roleLabel":"Warden","locked":false,"takenBy":""},{"id":"oarsman","name":"Oarsman","portrait":0,"icon":"skull","role":"tank","roleLabel":"Bulwark","locked":true,"takenBy":""}],"players":[{"id":"p1","slot":1,"at":"vex","ready":false},{"id":"p2","slot":2,"at":"sela","ready":true}]}'>
<meta name="fixture:empty" content='{"title":"","selected":"","characters":[],"players":[]}'>
<meta name="fixture:none" content='{"title":"Ranked · Pick your fighter","selected":"","characters":[{"id":"brenna","name":"Brenna","portrait":0,"icon":"shield","role":"tank","roleLabel":"Vanguard","locked":false,"takenBy":""},{"id":"vex","name":"Vex","portrait":0,"icon":"sword","role":"damage","roleLabel":"Striker","locked":false,"takenBy":""},{"id":"sela","name":"Sela","portrait":0,"icon":"heart","role":"support","roleLabel":"Mender","locked":false,"takenBy":""}],"players":[]}'>
<meta name="fixture:full" content='{"title":"","selected":"brenna","characters":[{"id":"brenna","name":"Brenna","portrait":0,"icon":"shield","role":"tank","roleLabel":"Vanguard","locked":false,"takenBy":""},{"id":"vex","name":"Vex","portrait":0,"icon":"sword","role":"damage","roleLabel":"Striker","locked":false,"takenBy":""},{"id":"sela","name":"Sela","portrait":0,"icon":"heart","role":"support","roleLabel":"Mender","locked":false,"takenBy":""},{"id":"idris","name":"Idris","portrait":0,"icon":"bolt","role":"damage","roleLabel":"Arcanist","locked":false,"takenBy":""}],"players":[{"id":"p1","slot":1,"at":"brenna","ready":true},{"id":"p2","slot":2,"at":"brenna","ready":false},{"id":"p3","slot":3,"at":"sela","ready":true},{"id":"p4","slot":4,"at":"idris","ready":true}]}'>
<meta name="fixture:disabled" content='{"title":"","selected":"","characters":[{"id":"brenna","name":"Brenna","portrait":0,"icon":"shield","role":"tank","roleLabel":"Vanguard","locked":false,"takenBy":"Kai"},{"id":"vex","name":"Vex","portrait":0,"icon":"sword","role":"damage","roleLabel":"Striker","locked":true,"takenBy":""},{"id":"sela","name":"Sela","portrait":0,"icon":"heart","role":"support","roleLabel":"Mender","locked":false,"takenBy":"Ana"},{"id":"idris","name":"Idris","portrait":0,"icon":"bolt","role":"damage","roleLabel":"Arcanist","locked":true,"takenBy":""}],"players":[{"id":"p1","slot":1,"at":"","ready":false}]}'>
<meta name="fixture:overflow" content='{"title":"Championship Series · Grand Finals · Choose your champion for the deciding match","selected":"c7","characters":[{"id":"c1","name":"Brenna","portrait":0,"icon":"shield","role":"tank","roleLabel":"Vanguard","locked":false,"takenBy":""},{"id":"c2","name":"Vex","portrait":0,"icon":"sword","role":"damage","roleLabel":"Striker","locked":false,"takenBy":""},{"id":"c3","name":"Sela","portrait":0,"icon":"heart","role":"support","roleLabel":"Mender","locked":false,"takenBy":"Maximilian Vandenberghe"},{"id":"c4","name":"Idris","portrait":0,"icon":"bolt","role":"damage","roleLabel":"Arcanist","locked":false,"takenBy":""},{"id":"c5","name":"Maren","portrait":0,"icon":"leaf","role":"support","roleLabel":"Warden","locked":false,"takenBy":""},{"id":"c6","name":"Oarsman","portrait":0,"icon":"skull","role":"tank","roleLabel":"Bulwark","locked":true,"takenBy":""},{"id":"c7","name":"Ysolde the Unbroken, Warden of the Ninefold Gate","portrait":0,"icon":"crown","role":"tank","roleLabel":"Guardian of the Eastern Marches","locked":false,"takenBy":""},{"id":"c8","name":"Lurker","portrait":0,"icon":"drop","role":"damage","roleLabel":"Assassin","locked":false,"takenBy":""},{"id":"c9","name":"Ashen Pyre","portrait":0,"icon":"fire","role":"damage","roleLabel":"Pyromancer","locked":true,"takenBy":""},{"id":"c10","name":"Tamsin","portrait":0,"icon":"music","role":"support","roleLabel":"Bard","locked":false,"takenBy":""},{"id":"c11","name":"Gorrum","portrait":0,"icon":"bomb","role":"tank","roleLabel":"Sapper","locked":false,"takenBy":""},{"id":"c12","name":"Quill","portrait":0,"icon":"star","role":"support","roleLabel":"Oracle","locked":false,"takenBy":""},{"id":"c13","name":"Rook","portrait":0,"icon":"key","role":"damage","roleLabel":"Duelist","locked":false,"takenBy":""},{"id":"c14","name":"Hollow","portrait":0,"icon":"gem","role":"damage","roleLabel":"Hexer","locked":true,"takenBy":""},{"id":"c15","name":"Wren","portrait":0,"icon":"apple","role":"support","roleLabel":"Herbalist","locked":false,"takenBy":""},{"id":"c16","name":"Kestrel","portrait":0,"icon":"flag","role":"damage","roleLabel":"Ranger","locked":false,"takenBy":""}],"players":[{"id":"p1","slot":1,"at":"c7","ready":false},{"id":"p2","slot":2,"at":"c7","ready":true},{"id":"p3","slot":3,"at":"c7","ready":false},{"id":"p4","slot":4,"at":"c7","ready":true}]}'>

<!--
  Character selection: a heading, a grid of character cells (portrait or
  icon, name, the role in words) and, on each cell, the marks of the
  players standing on it (P1…P4, a check once ready). A free cell reports
  choose with its key; a locked or taken one is disabled and names why
  (Locked, Taken by …). The local player's character is aria-selected and
  named under the grid. The game decides what can be picked; the roster
  only reports the choice.
-->
<section id="ArenaRoster" part="ArenaRoster.frame" aria-label={title != '' ? title : t("arena.roster.title")}>
  <header part="ArenaRoster.band"><h2 part="ArenaRoster.title">{title != '' ? title : t("arena.roster.title")}</h2></header>
  {#if characters.length}
    <scroll class="grid-scroll">
      <div class="grid" role="listbox" aria-label={t("arena.roster.characters")}>
        {#each characters as c (c.id)}
          <button part="ArenaRoster.cell" role="option" aria-selected={c.id == selected} aria-label={c.name} aria-description={c.locked ? t("arena.roster.locked") : (c.takenBy != '' ? t("arena.roster.taken", { name: c.takenBy }) : c.roleLabel)} data-role={c.role} data-locked={c.locked} data-taken={c.takenBy != ''} disabled={c.locked || c.takenBy != ''} onclick={choose}><div class="face">
            <div part="ArenaRoster.portrait" class="portrait">
              <img class="art" src={c.portrait} alt="" hidden={!c.portrait}>
              <og-icon class="glyph" icon={c.icon} hidden={!!c.portrait}></og-icon>
              <div class="marks">
                {#each players as p (p.id)}
                  {#if p.at == c.id}
                    <div part="ArenaRoster.mark" class="mark" data-slot={p.slot} data-ready={p.ready} aria-label={p.ready ? t("arena.roster.slotReady", { slot: p.slot }) : t("arena.roster.slotName", { slot: p.slot })}><span>{t("arena.roster.slot", { slot: p.slot })}</span><og-icon class="check" icon="check" hidden={!p.ready}></og-icon></div>
                  {/if}
                {/each}
              </div>
            </div>
            <span part="ArenaRoster.name" class="name">{c.name}</span>
            <span part="ArenaRoster.role" class="role" data-role={c.role}>{c.roleLabel}</span>
            <span part="ArenaRoster.badge" class="tag" data-tag={c.locked ? 'locked' : 'taken'} hidden={!(c.locked || c.takenBy != '')}>{c.locked ? t("arena.roster.locked") : t("arena.roster.taken", { name: c.takenBy })}</span>
          </div></button>
        {/each}
      </div>
    </scroll>
    <footer class="chosen" role="status" hidden={selected == ''}>
      {#each characters as c (c.id)}
        {#if c.id == selected}
          <span part="ArenaRoster.badge" class="tag" data-tag="selected">{t("arena.roster.chosen")}</span>
          <span part="ArenaRoster.name" class="chosen-name">{c.name}</span>
          <span part="ArenaRoster.role" class="chosen-role" data-role={c.role}>{c.roleLabel}</span>
        {/if}
      {/each}
    </footer>
  {:else}
    <p class="empty-text" role="status">{t("arena.roster.empty")}</p>
  {/if}
</section>

<style>
[part="ArenaRoster.frame"][hidden], [part="ArenaRoster.frame"] [hidden] { display: none; }
[part="ArenaRoster.frame"] { display: flex; flex-direction: column; width: stretch; min-width: 0; max-height: stretch; gap: var(--space-sm); container-type: inline-size; }
[part="ArenaRoster.band"] { display: flex; width: stretch; align-items: center; flex: none; }
[part="ArenaRoster.title"] { margin: 0; min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.grid-scroll { width: stretch; flex: 1; min-height: 0; }
.grid { display: flex; flex-wrap: wrap; justify-content: center; gap: var(--space-sm); width: stretch; padding: var(--space-xxs); }
[part="ArenaRoster.cell"] { flex: none; width: calc(var(--size-xxl) * 1.25); padding: var(--space-xs); align-items: stretch; justify-content: flex-start; }
[part="ArenaRoster.cell"] .face { display: flex; flex-direction: column; width: stretch; gap: var(--space-xxs); }
[part="ArenaRoster.portrait"] { position: relative; display: flex; width: stretch; aspect-ratio: 1; align-items: center; justify-content: center; overflow: hidden; }
.art { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.glyph { width: var(--size-lg); height: var(--size-lg); }
.marks { position: absolute; left: var(--space-xxs); right: var(--space-xxs); top: var(--space-xxs); display: flex; flex-wrap: wrap; gap: var(--space-xxs); }
[part="ArenaRoster.mark"] { display: flex; align-items: center; gap: var(--space-xxs); padding: 0 var(--space-xs); white-space: nowrap; }
.check { width: var(--size-xs); height: var(--size-xs); }
.name, .role { width: stretch; text-align: center; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
/* A badge wraps rather than lose its reason (Taken by …). */
.tag { max-width: stretch; align-self: center; padding: 0 var(--space-xs); text-align: center; overflow-wrap: anywhere; }
.chosen { display: flex; flex: none; width: stretch; align-items: baseline; justify-content: center; gap: var(--space-sm); min-width: 0; }
.chosen .tag { align-self: center; }
.chosen-name { min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.chosen-role { flex: none; white-space: nowrap; }
.empty-text { margin: 0; padding: var(--space-lg); text-align: center; }
/* The roster sits in a panel: its own width, not the screen's, tightens it,
   once three whole cells no longer fit on a line. */
@container (width < 380px) {
  .grid { gap: var(--space-xs); }
  [part="ArenaRoster.cell"] { width: calc(var(--size-xxl) * 0.95); padding: var(--space-xxs); }
  .glyph { width: var(--size-md); height: var(--size-md); }
  .chosen { flex-wrap: wrap; gap: var(--space-xs); }
}
</style>
