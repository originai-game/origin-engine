<meta name="component" content="ArenaRoster">
<meta name="strings" content="./arena.en.ogt">
<meta name="parts" content="ArenaRoster.frame ArenaRoster.band ArenaRoster.title ArenaRoster.cell ArenaRoster.portrait ArenaRoster.name ArenaRoster.role ArenaRoster.badge ArenaRoster.mark ArenaRoster.picture ArenaRoster.label ArenaRoster.headline ArenaRoster.caption ArenaRoster.well ArenaRoster.fade">
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
<section id="ArenaRoster" part="ArenaRoster.frame" data-layout="shell" data-gap="loose" aria-label={title != '' ? title : t("arena.roster.title")}>
  <header part="ArenaRoster.band" data-layout="row"><h2 part="ArenaRoster.title">{title != '' ? title : t("arena.roster.title")}</h2></header>
  {#if characters.length}
    <div class="grid-shell" data-layout="stack" data-gap="none" data-fill>
    <div class="grid-scroll" data-layout="scroll" data-fill>
      <div class="grid" data-layout="grid" role="listbox" aria-label={t("arena.roster.characters")}>
        {#each characters as c (c.id)}
          <button part="ArenaRoster.cell" role="option" aria-selected={c.id == selected} aria-label={c.name} aria-description={c.locked ? t("arena.roster.locked") : (c.takenBy ? t("arena.roster.taken", { name: c.takenBy }) : c.roleLabel)} data-role={c.role} data-locked={c.locked} data-taken={!!c.takenBy} disabled={c.locked || !!c.takenBy} onclick={choose}><div class="face" data-layout="stack" data-gap="hair">
            <div part="ArenaRoster.portrait ArenaRoster.picture" class="portrait" data-layout="ratio">
              {#if c.portrait}<img class="art" src={c.portrait} alt="">{:else}<og-icon class="glyph" icon={c.icon}></og-icon>{/if}
              <div class="marks" data-layout="cluster" data-gap="hair">
                {#each players as p (p.id)}
                  {#if p.at == c.id}
                    <div part="ArenaRoster.mark ArenaRoster.badge" class="mark" data-layout="row" data-gap="hair" data-slot={p.slot} data-ready={p.ready} aria-label={p.ready ? t("arena.roster.slotReady", { slot: p.slot }) : t("arena.roster.slotName", { slot: p.slot })}><span>{t("arena.roster.slot", { slot: p.slot })}</span>{#if p.ready}<og-icon class="check" icon="check"></og-icon>{/if}</div>
                  {/if}
                {/each}
              </div>
            </div>
            <span part="ArenaRoster.name ArenaRoster.headline" class="name">{c.name}</span>
            <span part="ArenaRoster.role ArenaRoster.label ArenaRoster.caption" class="role" data-role={c.role}>{c.roleLabel}</span>
            {#if c.locked || c.takenBy}<span part="ArenaRoster.badge" class="tag" data-tag={c.locked ? 'locked' : 'taken'}>{c.locked ? t("arena.roster.locked") : t("arena.roster.taken", { name: c.takenBy })}</span>{/if}
          </div></button>
        {/each}
      </div>
    </div>
    <div class="scroll-fade" part="ArenaRoster.fade" data-anchor="bottom-edge"></div>
    </div>
    {#if selected != ''}
    <footer class="chosen" part="ArenaRoster.well" data-layout="cluster" role="status">
      {#each characters as c (c.id)}
        {#if c.id == selected}
          <span part="ArenaRoster.badge" class="tag" data-tag="selected">{t("arena.roster.chosen")}</span>
          <span part="ArenaRoster.name" class="chosen-name">{c.name}</span>
          <span part="ArenaRoster.role ArenaRoster.label" class="chosen-role" data-role={c.role}>{c.roleLabel}</span>
        {/if}
      {/each}
    </footer>
    {/if}
  {:else}
    <p class="empty-text" part="ArenaRoster.caption" role="status">{t("arena.roster.empty")}</p>
  {/if}
</section>

<style>
/* Character selection on a smoked cut card: each character a tile of dark
   glass with its portrait shaded into its foot, the chosen one ringed in
   the accent and washed in it; a locked or taken one drawn grey with its
   reason on a tag. Names wrap on whole words and are never cut. */
:host { inline-size: stretch; min-inline-size: 0; max-block-size: stretch; color: var(--hud-ink); --fade-fill: var(--party-open-fill); }
[part="ArenaRoster.frame"] { box-sizing: border-box; padding: var(--space-lg); border: 0; border-radius: 0; color: var(--hud-ink); background: var(--party-card-fill); box-shadow: var(--party-card-rim); clip-path: polygon(var(--party-cut) 0, 100% 0, 100% calc(100% - var(--party-cut)), calc(100% - var(--party-cut)) 100%, 0 100%, 0 var(--party-cut)); }
[part="ArenaRoster.band"] { width: stretch; flex: none; min-height: 0; padding: 0; border: 0; border-radius: 0; background: none; box-shadow: none; }
[part="ArenaRoster.title"] { margin: 0; min-inline-size: 0; font-size: var(--hud-size-md); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: none; white-space: normal; overflow-wrap: break-word; }
.grid-shell { position: relative; min-block-size: 0; }
.grid-scroll { overflow-y: auto; flex: 1; min-height: 0; }
.grid { --layout-tile: min(var(--party-tile), 45cqi); padding: var(--space-xxs); padding-block-end: var(--space-lg); }
[part="ArenaRoster.cell"] { padding: 0; align-items: stretch; justify-content: flex-start; border: 0; border-radius: 0; color: var(--hud-ink); background: var(--party-tile-fill); box-shadow: var(--party-tile-rim); clip-path: polygon(var(--party-cut) 0, 100% 0, 100% calc(100% - var(--party-cut)), calc(100% - var(--party-cut)) 100%, 0 100%, 0 var(--party-cut)); }
[part="ArenaRoster.cell"]:hover { box-shadow: var(--party-tile-hover-rim); }
[part="ArenaRoster.cell"]:focus-visible { box-shadow: var(--party-focus-rim); }
[part="ArenaRoster.cell"][aria-selected=true] { background: var(--party-tile-on-fill); box-shadow: var(--party-on-rim); }
[part="ArenaRoster.cell"]:disabled { color: var(--hud-ink-dim); background: var(--party-tile-fill); box-shadow: var(--party-tile-rim); }
[part="ArenaRoster.cell"] .face { width: stretch; height: stretch; padding-block-end: var(--space-sm); }
[part="ArenaRoster.portrait ArenaRoster.picture"] { --layout-ratio: 1.15; position: relative; width: stretch; overflow: hidden; border: 0; border-radius: 0; background: none; box-shadow: none; }
.art { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.glyph { width: var(--size-lg); height: var(--size-lg); color: var(--hud-ink); }
[part="ArenaRoster.cell"]:disabled .art { filter: var(--party-out-art); opacity: 1; }
[part="ArenaRoster.cell"]:disabled .glyph { color: var(--hud-ink-dim); }
.marks { position: absolute; left: var(--space-xs); right: var(--space-xs); top: var(--space-xs); }
[part~="ArenaRoster.mark"] { padding: 0 var(--space-xs); white-space: nowrap; font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); color: var(--hud-key-ink); background: var(--hud-key-fill); box-shadow: var(--hud-key-shadow); border-radius: var(--hud-key-radius); }
[part~="ArenaRoster.mark"][data-ready=true] { color: var(--party-ready-ink); background: var(--party-ready-fill); }
.check { width: var(--size-xs); height: var(--size-xs); }
.name { padding-inline: var(--space-xs); text-align: center; font-size: var(--hud-size-name); line-height: 1.15; font-weight: var(--hud-figure-weight); color: inherit; overflow-wrap: break-word; }
.role { padding-inline: var(--space-xs); text-align: center; overflow-wrap: break-word; font-size: var(--hud-label-size); line-height: 1.3; font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
/* A badge wraps rather than lose its reason (Taken by …). */
.tag { flex: none; padding: 0 var(--space-xs); border: 0; border-radius: 0; font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); text-shadow: none; clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); max-width: stretch; align-self: center; text-align: center; overflow-wrap: anywhere; }
.tag[data-tag=selected] { color: var(--party-go-ink); background: var(--hud-accent); box-shadow: none; }
.tag[data-tag=locked], .tag[data-tag=taken] { color: var(--hud-ink-dim); }
.chosen { flex: none; align-items: center; justify-content: flex-start; padding: var(--space-sm) var(--space-md); border-radius: 0; background: var(--hud-plate-fill); box-shadow: var(--hud-plate-lit-rim); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
.chosen-name { min-width: 0; font-size: var(--party-size-name); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); text-transform: var(--hud-label-transform); color: var(--hud-ink); overflow-wrap: break-word; }
/* A role longer than the footer wraps inside it. */
.chosen-role { flex: 0 1 auto; min-width: 0; max-inline-size: 100%; overflow-wrap: anywhere; font-size: var(--hud-label-size); line-height: 1.3; font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
.empty-text { margin: 0; padding: var(--space-lg); text-align: center; font-size: var(--hud-size-sm); color: var(--hud-ink-dim); }
/* The roster sits in a panel: its own width, not the screen's, tightens it. */
@container (width < 380px) {
  [part="ArenaRoster.frame"] { padding: var(--space-sm); }
  .grid { --layout-tile: min(var(--party-tile-compact), 45cqi); --layout-gap: var(--layout-gap-tight); }
  .glyph { width: var(--size-md); height: var(--size-md); }
  .chosen { --layout-gap: var(--layout-gap-tight); padding: var(--space-xs) var(--space-sm); }
  .chosen-name { font-size: var(--party-size-name-compact); }
}
</style>
