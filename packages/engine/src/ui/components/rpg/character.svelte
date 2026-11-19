<meta name="component" content="RpgCharacter">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgCharacter.frame RpgCharacter.band RpgCharacter.title RpgCharacter.name RpgCharacter.cell RpgCharacter.row RpgCharacter.caption RpgCharacter.figure">
<meta name="prop:title" content="string" data-doc="A heading band over the figure (Equipment); empty shows the name header instead.">
<meta name="prop:name" content="string" data-doc="The character's name; with an empty title it heads the sheet beside the portrait.">
<meta name="prop:line" content="string" data-doc="The class and level line as the game words it (Rogue · Level 12).">
<meta name="prop:portrait" content="number" data-doc="A head-and-shoulders texture (og.load.texture); 0 hides it.">
<meta name="prop:figure" content="number" data-doc="A full-figure texture shown between the slot columns; 0 leaves the ground.">
<meta name="prop:slots" content="list" data-doc="The equipment slots, keyed rows of { id, side, label, icon, rarity }: side is start or end (the column it stands in, mirrored right to left); an empty icon is an empty slot showing its label.">
<meta name="prop:selected" content="string" data-doc="The selected slot's id; empty is none.">
<meta name="prop:attributes" content="list" data-doc="The attributes, keyed rows of { id, group, icon, label, value }: group a or b, the two columns; empty hides them (a page with RpgStats beside it).">
<meta name="action:details" content="Unequip">
<meta name="fixture:default" content='{"title":"","name":"Brenna Vale","line":"Ranger · Level 12","portrait":0,"figure":0,"slots":[{"id":"hand","side":"start","label":"Weapon","icon":"sword","rarity":"legendary"},{"id":"neck","side":"end","label":"Amulet","icon":"","rarity":"common"}],"selected":"hand","attributes":[{"id":"vit","group":"a","icon":"heart","label":"Vitality","value":"142"},{"id":"str","group":"b","icon":"sword","label":"Strength","value":"14"}]}'>

<!--
  A character sheet: a portrait with name and class, two columns of
  equipment slots either side of the full figure, and the attributes in two
  columns beneath. The game owns what fits where; the sheet reports choose
  (a slot's key) and unequip (X or pad Y on the focused slot, its key).
-->
<section id="RpgCharacter" part="RpgCharacter.frame" data-layout="shell" aria-label={title ? title : name}>
  {#if title}<header part="RpgCharacter.band" data-layout="row" data-gap="loose"><div class="band-end start"></div><h2 part="RpgCharacter.title">{title}</h2><div class="band-end finish"></div></header>
  {:else}<header class="who" data-layout="row" data-gap="loose">
    {#if portrait}<img class="portrait" src={portrait} alt="">{/if}
    <div class="who-text" data-layout="stack" data-gap="hair" data-fill>
      <h2 part="RpgCharacter.name">{name}</h2>
      <p class="char-line" part="RpgCharacter.caption">{line}</p>
      <div class="rpg-rule" role="separator"></div>
    </div>
  </header>{/if}
  <div class="doll" data-layout="row" data-fill>
    {#if figure}<img class="figure" src={figure} alt="">{/if}
    <div class="column start" data-layout="stack" data-gap="tight" role="group">
      {#each slots as s (s.id)}
        {#if s.side == 'start'}
          <button part="RpgCharacter.cell" data-rarity={s.icon ? s.rarity : ''} data-empty={!s.icon} aria-selected={s.id == selected} aria-label={s.label} onclick={choose} ondetails={unequip}><div class="face" data-layout="cover"><og-icon class="art" icon={s.icon ? s.icon : 'plus'}></og-icon></div></button>
        {/if}
      {/each}
    </div>
    <div class="column end" data-layout="stack" data-gap="tight" role="group">
      {#each slots as s (s.id)}
        {#if s.side == 'end'}
          <button part="RpgCharacter.cell" data-rarity={s.icon ? s.rarity : ''} data-empty={!s.icon} aria-selected={s.id == selected} aria-label={s.label} onclick={choose} ondetails={unequip}><div class="face" data-layout="cover"><og-icon class="art" icon={s.icon ? s.icon : 'plus'}></og-icon></div></button>
        {/if}
      {/each}
    </div>
  </div>
  {#if attributes.length}<div class="attributes" data-layout="switcher" data-gap="loose" role="table" aria-label={t("rpg.character.attributes")}>
    <div class="attr-column" data-layout="stack" data-gap="none" data-fill>
      {#each attributes as a (a.id)}
        {#if a.group == 'a'}<div part="RpgCharacter.row" data-layout="row" data-gap="tight" role="row"><og-icon class="attr-mark" icon={a.icon}></og-icon><span class="attr-label" data-fill role="rowheader">{a.label}</span><span class="attr-value" part="RpgCharacter.figure" role="cell">{a.value}</span></div>{/if}
      {/each}
    </div>
    <hr class="attr-split" vertical>
    <div class="attr-column" data-layout="stack" data-gap="none" data-fill>
      {#each attributes as a (a.id)}
        {#if a.group == 'b'}<div part="RpgCharacter.row" data-layout="row" data-gap="tight" role="row"><og-icon class="attr-mark" icon={a.icon}></og-icon><span class="attr-label" data-fill role="rowheader">{a.label}</span><span class="attr-value" part="RpgCharacter.figure" role="cell">{a.value}</span></div>{/if}
      {/each}
    </div>
  </div>{/if}
</section>

<style>
/* The character sheet on a smoked cut card: an octagon portrait beside
   the name in condensed capitals, the figure standing in a glow between
   two columns of cut slots (each in its rarity, the chosen one ringed in
   the accent, an empty one a dark well), the attributes on ruled rows. */
:host { color: var(--hud-ink); }
[part="RpgCharacter.frame"] { box-sizing: border-box; padding: var(--space-lg); border: 0; border-radius: 0; color: var(--hud-ink); background: var(--party-card-fill); box-shadow: var(--party-card-rim); clip-path: polygon(var(--party-cut) 0, 100% 0, 100% calc(100% - var(--party-cut)), calc(100% - var(--party-cut)) 100%, 0 100%, 0 var(--party-cut)); }
[part="RpgCharacter.band"] { width: stretch; justify-content: center; flex: none; min-height: 0; padding: 0; border: 0; border-radius: 0; background: none; box-shadow: none; }
[part="RpgCharacter.title"] { margin: 0; min-inline-size: 0; font-size: var(--hud-size-md); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: none; white-space: normal; overflow-wrap: break-word; text-align: center; }
.band-end { width: var(--size-lg); height: var(--space-lg); flex: none; }
.who { width: stretch; flex: none; padding-block-end: var(--space-sm); box-shadow: var(--party-rule); }
.portrait { width: var(--size-xl); height: var(--size-xl); object-fit: cover; object-position: center top; flex: none; clip-path: var(--hud-octagon); }
.who-text { min-width: 0; align-items: flex-start; text-align: start; }
[part="RpgCharacter.name"] { margin: 0; max-width: stretch; font-size: var(--party-size-name); line-height: 1.05; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); overflow-wrap: break-word; }
.char-line { margin: 0; max-width: stretch; font-size: var(--hud-label-size); line-height: 1.3; font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); overflow-wrap: break-word; }
.doll { position: relative; width: stretch; min-height: 0; justify-content: space-between; padding: var(--space-sm) var(--space-xs); background: var(--party-figure-glow); }
.figure { position: absolute; inset-inline-start: 18%; top: 0; bottom: 0; inline-size: 64%; block-size: 100%; object-fit: contain; object-position: center bottom; filter: var(--party-figure-shadow); }
.column { position: relative; height: stretch; min-height: 0; justify-content: space-between; }
[part="RpgCharacter.cell"] { flex: none; width: var(--party-slot); height: var(--party-slot); padding: var(--space-xs); border: 0; border-radius: 0; color: var(--hud-ink); background: var(--party-tile-fill); box-shadow: var(--party-tile-rim); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
[part="RpgCharacter.cell"]:hover { box-shadow: var(--party-tile-hover-rim); }
[part="RpgCharacter.cell"][data-rarity=common] { background: var(--cell-common-fill); box-shadow: var(--cell-common-shadow); }
[part="RpgCharacter.cell"][data-rarity=uncommon] { background: var(--cell-uncommon-fill); box-shadow: var(--cell-uncommon-shadow); }
[part="RpgCharacter.cell"][data-rarity=rare] { background: var(--cell-rare-fill); box-shadow: var(--cell-rare-shadow); }
[part="RpgCharacter.cell"][data-rarity=epic] { background: var(--cell-epic-fill); box-shadow: var(--cell-epic-shadow); }
[part="RpgCharacter.cell"][data-rarity=legendary] { background: var(--cell-legendary-fill); box-shadow: var(--cell-legendary-shadow); }
[part="RpgCharacter.cell"][data-rarity=mythic] { background: var(--cell-mythic-fill); box-shadow: var(--cell-mythic-shadow); }
[part="RpgCharacter.cell"][data-empty=true] { color: var(--hud-ink-dim); background: var(--party-open-fill); box-shadow: var(--party-seat-rim); }
[part="RpgCharacter.cell"][aria-selected=true] { box-shadow: var(--party-on-rim); }
[part="RpgCharacter.cell"]:focus-visible { box-shadow: var(--party-focus-rim); }
[part="RpgCharacter.cell"] .face { width: stretch; height: stretch; }
.art { width: stretch; height: stretch; }
[data-empty=true] .art { width: 50%; height: 50%; }
.attributes { flex: none; padding: var(--space-sm) 0 0; box-shadow: var(--party-rule-over); --layout-switch-at: var(--measure-xs); }
.attr-column { align-self: stretch; min-width: 0; }
.attr-split { width: var(--stroke-hairline); align-self: stretch; margin: 0; }
[part="RpgCharacter.row"] { width: stretch; padding: var(--space-xs) 0; border-radius: 0; color: var(--hud-ink); background: none; box-shadow: var(--party-rule); }
.attr-mark { width: var(--size-xs); height: var(--size-xs); flex: none; color: var(--hud-label-ink); }
.attr-label { min-width: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-label-weight); color: var(--hud-label-ink); overflow-wrap: break-word; }
.attr-value { flex: none; white-space: nowrap; font-size: var(--hud-size-readout); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); text-shadow: none; }
@media (width < 600px) {
  [part="RpgCharacter.frame"] { padding: var(--space-sm); }
  /* Five slots a column fit the doll's height on a phone at any text size. */
  [part="RpgCharacter.cell"] { width: var(--size-touch); height: var(--size-touch); padding: var(--space-xxs); }
  .portrait { width: var(--size-lg); height: var(--size-lg); }
  [part="RpgCharacter.name"] { font-size: var(--party-size-name-compact); }
}
@container (width < 300px) { .attr-split { display: none; } }
@media (height < 500px) { .doll { padding: var(--space-xxs) var(--space-xs); } .column { --layout-gap-tight: var(--layout-gap-hair); } [part="RpgCharacter.cell"] { width: var(--size-lg); height: var(--size-lg); padding: var(--space-xxs); } }
/* A short screen (a desktop at large text): smaller slots, so the doll can give height to the attributes. */
@media (600px <= width) and (500px <= height < 800px) { [part="RpgCharacter.cell"] { width: var(--size-lg); height: var(--size-lg); padding: var(--space-xxs); } .column { --layout-gap-tight: var(--layout-gap-hair); } }
@media (600px <= width) and (500px <= height < 800px) { .attr-mark { display: none; } .attributes { --layout-gap-loose: var(--layout-gap); } }
</style>
