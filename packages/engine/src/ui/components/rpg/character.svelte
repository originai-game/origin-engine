<meta name="component" content="RpgCharacter">
<meta name="strings" content="./rpg.en.ogt">
<meta name="prop:title" content="string" data-doc="A heading band over the figure (Equipment); empty shows the name header instead.">
<meta name="prop:name" content="string" data-doc="The character's name; with an empty title it heads the sheet beside the portrait.">
<meta name="prop:line" content="string" data-doc="The class and level line as the game words it (Rogue · Level 12).">
<meta name="prop:portrait" content="number" data-doc="A head-and-shoulders texture (og.load.texture); 0 hides it.">
<meta name="prop:figure" content="number" data-doc="A full-figure texture shown between the slot columns; 0 leaves the ground.">
<meta name="prop:slots" content="list" data-doc="The equipment slots, keyed rows of { id, side, label, icon, rarity }: side is start or end (the column it stands in, mirrored right to left); an empty icon is an empty slot showing its label.">
<meta name="prop:selected" content="string" data-doc="The selected slot's id; empty is none.">
<meta name="prop:attributes" content="list" data-doc="The attributes, keyed rows of { id, group, icon, label, value }: group a or b, the two columns; empty hides them (a page with RpgStats beside it).">
<meta name="prop:compact" content="bool" data-doc="Its place is narrow (under about 300 px): the attributes (their split goes) tighten. The place decides, not the screen.">
<meta name="prop:dense" content="bool" data-doc="A wide but short place (a laptop window): smaller slots and tighter attributes.">
<meta name="action:details" content="Unequip">
<meta name="fixture:default" content='{"title":"","name":"Brenna Vale","line":"Ranger · Level 12","portrait":0,"figure":0,"slots":[{"id":"hand","side":"start","label":"Weapon","icon":"sword","rarity":"legendary"},{"id":"neck","side":"end","label":"Amulet","icon":"","rarity":"common"}],"selected":"hand","attributes":[{"id":"vit","group":"a","icon":"heart","label":"Vitality","value":"142"},{"id":"str","group":"b","icon":"sword","label":"Strength","value":"14"}]}'>

<!--
  A character sheet: a portrait with name and class, two columns of
  equipment slots either side of the full figure, and the attributes in two
  columns beneath. The game owns what fits where; the sheet reports choose
  (a slot's key) and unequip (X or pad Y on the focused slot, its key).
-->
<section class="frame" id="RpgCharacter" part="frame" layout="stack" fill aria-label={title ? title : name} class:narrow={compact} class:dense={dense}>
  {#if title}<header class="band" part="band" layout="row" gap="loose"><div class="band-end start"></div><h2 class="title" part="title">{title}</h2><div class="band-end finish"></div></header>
  {:else}<header class="who" layout="row" gap="loose">
    {#if portrait}<img class="portrait" src={portrait} alt="">{/if}
    <div class="who-text" layout="stack" gap="hair" fill>
      <h2 class="name" part="name">{name}</h2>
      <p class="char-line" part="caption">{line}</p>
      <div class="rpg-rule" role="separator"></div>
    </div>
  </header>{/if}
  <div class="doll" layout="row" fill>
    {#if figure}<img class="figure" src={figure} alt="">{/if}
    <div class="column start" layout="stack" gap="tight" role="group">
      {#each slots as s (s.id)}
        {#if s.side == 'start'}
          <button class="cell" part="cell" tier={s.icon ? s.rarity : ''} aria-selected={s.id == selected} aria-label={s.label} onclick={choose} ondetails={unequip} state:empty={!s.icon} state:selected={s.id == selected}><div class="face cover" layout="stack" fill><og-icon class="art" icon={s.icon ? s.icon : 'plus'}></og-icon></div></button>
        {/if}
      {/each}
    </div>
    <div class="column end" layout="stack" gap="tight" role="group">
      {#each slots as s (s.id)}
        {#if s.side == 'end'}
          <button class="cell" part="cell" tier={s.icon ? s.rarity : ''} aria-selected={s.id == selected} aria-label={s.label} onclick={choose} ondetails={unequip} state:empty={!s.icon} state:selected={s.id == selected}><div class="face cover" layout="stack" fill><og-icon class="art" icon={s.icon ? s.icon : 'plus'}></og-icon></div></button>
        {/if}
      {/each}
    </div>
  </div>
  {#if attributes.length}<div class="attributes" layout="wrap" gap="loose" role="table" aria-label={t("rpg.character.attributes")}>
    <div class="attr-column" layout="stack" gap="none" fill>
      {#each attributes as a (a.id)}
        {#if a.group == 'a'}<div class="row" part="row" layout="row" gap="tight" role="row"><og-icon class="attr-mark" icon={a.icon}></og-icon><span class="attr-label" fill role="rowheader">{a.label}</span><span class="attr-value" part="figure" role="cell">{a.value}</span></div>{/if}
      {/each}
    </div>
    <hr class="attr-split" vertical>
    <div class="attr-column" layout="stack" gap="none" fill>
      {#each attributes as a (a.id)}
        {#if a.group == 'b'}<div class="row" part="row" layout="row" gap="tight" role="row"><og-icon class="attr-mark" icon={a.icon}></og-icon><span class="attr-label" fill role="rowheader">{a.label}</span><span class="attr-value" part="figure" role="cell">{a.value}</span></div>{/if}
      {/each}
    </div>
  </div>{/if}
</section>

<style>
.cover { justify-content: center; align-items: center; }
/* The character sheet on a smoked cut card: an octagon portrait beside
   the name in condensed capitals, the figure standing in a glow between
   two columns of cut slots (each in its rarity, the chosen one ringed in
   the accent, an empty one a dark well), the attributes on ruled rows. */
#RpgCharacter { color: var(--hud-ink); }
.frame { box-sizing: border-box; padding: var(--space-lg); border: 0; color: var(--hud-ink); background: var(--party-card-fill); box-shadow: var(--party-card-rim); border-radius: var(--party-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.band { width: stretch; justify-content: center; flex: none; min-height: 0; padding: 0; border: 0; border-radius: 0; background: none; box-shadow: none; }
.title { margin: 0; min-inline-size: 0; font-size: var(--hud-size-md); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: none; white-space: normal; overflow-wrap: break-word; text-align: center; }
.band-end { width: var(--size-lg); height: var(--space-lg); flex: none; }
.who { width: stretch; flex: none; padding-block-end: var(--space-sm); box-shadow: var(--party-rule); }
.portrait { width: var(--size-xl); height: var(--size-xl); object-fit: cover; object-position: center top; flex: none; clip-path: var(--hud-octagon); }
.who-text { min-width: 0; align-items: flex-start; text-align: start; }
.name { margin: 0; max-width: stretch; font-size: var(--party-size-name); line-height: 1.05; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); overflow-wrap: break-word; }
.char-line { margin: 0; max-width: stretch; font-size: var(--hud-label-size); line-height: 1.3; font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); overflow-wrap: break-word; }
.doll { position: relative; width: stretch; min-height: 0; justify-content: space-between; padding: var(--space-sm) var(--space-xs); background: var(--party-figure-glow); }
.figure { position: absolute; inset-inline-start: 18%; top: 0; bottom: 0; inline-size: 64%; block-size: 100%; object-fit: contain; object-position: center bottom; filter: var(--party-figure-shadow); }
.column { position: relative; height: stretch; min-height: 0; justify-content: space-between; }
.cell { flex: none; width: var(--party-slot); height: var(--party-slot); padding: var(--space-xs); border: 0; color: var(--hud-ink); background: var(--party-tile-fill); box-shadow: var(--party-tile-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.cell:hover { box-shadow: var(--party-tile-hover-rim); }
.cell.rarity-common { background: var(--cell-common-fill); box-shadow: var(--cell-common-shadow); }
.cell.rarity-uncommon { background: var(--cell-uncommon-fill); box-shadow: var(--cell-uncommon-shadow); }
.cell.rarity-rare { background: var(--cell-rare-fill); box-shadow: var(--cell-rare-shadow); }
.cell.rarity-epic { background: var(--cell-epic-fill); box-shadow: var(--cell-epic-shadow); }
.cell.rarity-legendary { background: var(--cell-legendary-fill); box-shadow: var(--cell-legendary-shadow); }
.cell.rarity-mythic { background: var(--cell-mythic-fill); box-shadow: var(--cell-mythic-shadow); }
.cell:state(empty) { color: var(--hud-ink-dim); background: var(--party-open-fill); box-shadow: var(--party-seat-rim); }
.cell:state(selected) { box-shadow: var(--party-on-rim); }
.cell:focus-visible { box-shadow: var(--party-focus-rim); }
.cell .face { width: stretch; height: stretch; }
.art { width: stretch; height: stretch; }
:state(empty) .art { width: 50%; height: 50%; }
.attributes { flex: none; padding: var(--space-sm) 0 0; box-shadow: var(--party-rule-over); --layout-switch-at: var(--measure-xs); }
.attr-column { align-self: stretch; min-width: 0; }
.attr-split { width: var(--stroke-hairline); align-self: stretch; margin: 0; }
.row { width: stretch; padding: var(--space-xs) 0; border-radius: 0; color: var(--hud-ink); background: none; box-shadow: var(--party-rule); }
.attr-mark { width: var(--size-xs); height: var(--size-xs); flex: none; color: var(--hud-label-ink); }
.attr-label { min-width: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-label-weight); color: var(--hud-label-ink); overflow-wrap: break-word; }
.attr-value { flex: none; white-space: nowrap; font-size: var(--hud-size-readout); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); text-shadow: none; }
.frame:size(compact) { padding: var(--space-sm); }
.cell:size(compact) { width: var(--size-touch); height: var(--size-touch); padding: var(--space-xxs); }
.portrait:size(compact) { width: var(--size-lg); height: var(--size-lg); }
.name:size(compact) { font-size: var(--party-size-name-compact); }
#RpgCharacter.narrow .attr-split { display: none; }
.doll:size(short) { padding: var(--space-xxs) var(--space-xs); }
.column:size(short) { --layout-gap-tight: var(--layout-gap-hair); }
.cell:size(short) { width: var(--size-lg); height: var(--size-lg); padding: var(--space-xxs); }
/* A short screen (a desktop at large text): smaller slots, so the doll can give height to the attributes. */
#RpgCharacter.dense .cell { width: var(--size-lg); height: var(--size-lg); padding: var(--space-xxs); }
#RpgCharacter.dense .column { --layout-gap-tight: var(--layout-gap-hair); }
#RpgCharacter.dense .attr-mark { display: none; }
#RpgCharacter.dense .attributes { --layout-gap-loose: var(--layout-gap); }
</style>
