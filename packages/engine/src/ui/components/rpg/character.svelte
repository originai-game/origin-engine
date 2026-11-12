<meta name="component" content="RpgCharacter">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgCharacter.frame RpgCharacter.band RpgCharacter.title RpgCharacter.name RpgCharacter.cell RpgCharacter.row">
<meta name="prop:title" content="string" data-doc="A heading band over the figure (Equipment); empty shows the name header instead.">
<meta name="prop:name" content="string" data-doc="The character's name; with an empty title it heads the sheet beside the portrait.">
<meta name="prop:line" content="string" data-doc="The class and level line as the game words it (Rogue · Level 12).">
<meta name="prop:portrait" content="number" data-doc="A head-and-shoulders texture (og.load.texture); 0 hides it.">
<meta name="prop:figure" content="number" data-doc="A full-figure texture shown between the slot columns; 0 leaves the ground.">
<meta name="prop:slots" content="list" data-doc="The equipment slots, keyed rows of { id, side, label, icon, rarity }: side is start or end (the column it stands in, mirrored right to left); an empty icon is an empty slot showing its label.">
<meta name="prop:selected" content="string" data-doc="The selected slot's id; empty is none.">
<meta name="prop:attributes" content="list" data-doc="The attributes, keyed rows of { id, group, icon, label, value }: group a or b, the two columns; empty hides them (a page with RpgStats beside it).">
<meta name="action:details" content="Unequip">
<meta name="fixture:default" content='{"title":"","name":"Brenna Vale","line":"Ranger · Level 12","portrait":0,"figure":0,"slots":[{"id":"hand","side":"start","label":"Weapon","icon":"item-blade","rarity":"legendary"},{"id":"neck","side":"end","label":"Amulet","icon":"","rarity":"common"}],"selected":"hand","attributes":[{"id":"vit","group":"a","icon":"heart","label":"Vitality","value":"142"},{"id":"str","group":"b","icon":"sword","label":"Strength","value":"14"}]}'>

<!--
  A character sheet: a portrait with name and class, two columns of
  equipment slots either side of the full figure, and the attributes in two
  columns beneath. The game owns what fits where; the sheet reports choose
  (a slot's key) and unequip (X or pad Y on the focused slot, its key).
-->
<section id="RpgCharacter" part="RpgCharacter.frame" aria-label={title ? title : name}>
  <header part="RpgCharacter.band" hidden={!title}><div class="band-end start"></div><h2 part="RpgCharacter.title">{title}</h2><div class="band-end finish"></div></header>
  <header class="who" hidden={!!title}>
    <img class="portrait" src={portrait} alt="" hidden={!portrait}>
    <div class="who-text">
      <h2 part="RpgCharacter.name">{name}</h2>
      <p class="char-line">{line}</p>
      <div class="rpg-rule" role="separator"></div>
    </div>
  </header>
  <div class="doll">
    <img class="figure" src={figure} alt="" hidden={!figure}>
    <div class="column start" role="group">
      {#each slots as s (s.id)}
        {#if s.side == 'start'}
          <button part="RpgCharacter.cell" data-rarity={s.icon ? s.rarity : ''} data-empty={!s.icon} aria-selected={s.id == selected} aria-label={s.label} onclick={choose} ondetails={unequip}><div class="face"><og-icon class="art" icon={s.icon ? s.icon : 'plus'}></og-icon></div></button>
        {/if}
      {/each}
    </div>
    <div class="column end" role="group">
      {#each slots as s (s.id)}
        {#if s.side == 'end'}
          <button part="RpgCharacter.cell" data-rarity={s.icon ? s.rarity : ''} data-empty={!s.icon} aria-selected={s.id == selected} aria-label={s.label} onclick={choose} ondetails={unequip}><div class="face"><og-icon class="art" icon={s.icon ? s.icon : 'plus'}></og-icon></div></button>
        {/if}
      {/each}
    </div>
  </div>
  <div class="attributes" role="table" hidden={!attributes.length} aria-label={t("rpg.character.attributes")}>
    <div class="attr-column">
      {#each attributes as a (a.id)}
        {#if a.group == 'a'}<div part="RpgCharacter.row" role="row"><og-icon class="attr-mark" icon={a.icon}></og-icon><span class="attr-label" role="rowheader">{a.label}</span><span class="attr-value" role="cell">{a.value}</span></div>{/if}
      {/each}
    </div>
    <div class="attr-split"></div>
    <div class="attr-column">
      {#each attributes as a (a.id)}
        {#if a.group == 'b'}<div part="RpgCharacter.row" role="row"><og-icon class="attr-mark" icon={a.icon}></og-icon><span class="attr-label" role="rowheader">{a.label}</span><span class="attr-value" role="cell">{a.value}</span></div>{/if}
      {/each}
    </div>
  </div>
</section>

<style>
[part="RpgCharacter.frame"][hidden], [part="RpgCharacter.frame"] [hidden] { display: none; }
[part="RpgCharacter.frame"] { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-sm); container-type: inline-size; }
[part="RpgCharacter.band"] { display: flex; width: stretch; align-items: center; justify-content: center; gap: var(--space-md); flex: none; }
.band-end { width: var(--size-lg); height: var(--space-lg); flex: none; }
.who { display: flex; width: stretch; gap: var(--space-md); align-items: center; flex: none; }
.portrait { width: calc(var(--size-xxl) * 1.15); height: calc(var(--size-xxl) * 1.15); object-fit: cover; flex: none; }
.who-text { display: flex; flex-direction: column; width: stretch; gap: var(--space-xxs); align-items: center; text-align: center; }
[part="RpgCharacter.name"], .char-line { margin: 0; }
.doll { position: relative; display: flex; width: stretch; flex: 1; min-height: 0; justify-content: space-between; padding: var(--space-sm) var(--space-xs); }
.figure { position: absolute; left: 15%; top: 0; width: 70%; height: stretch; object-fit: contain; object-position: center bottom; }
.column { display: flex; flex-direction: column; gap: var(--space-xs); height: stretch; min-height: 0; justify-content: space-between; }
[part="RpgCharacter.cell"] { flex: none; width: var(--size-xl); height: var(--size-xl); padding: var(--space-xs); }
[part="RpgCharacter.cell"] .face { display: flex; width: stretch; height: stretch; align-items: center; justify-content: center; }
.art { width: stretch; height: stretch; }
.attributes { display: flex; width: stretch; gap: var(--space-md); flex: none; padding: var(--space-sm) var(--space-xs) 0; }
.attr-column { display: flex; flex-direction: column; width: stretch; }
.attr-split { width: var(--stroke-hairline); align-self: stretch; }
[part="RpgCharacter.row"] { display: flex; width: stretch; align-items: center; gap: var(--space-sm); padding: var(--space-xs) 0; }
.attr-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
.attr-label { width: stretch; min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
[part="RpgCharacter.row"] { column-gap: var(--space-xs); }
.attr-value { font-variant-numeric: tabular-nums; white-space: nowrap; }
@container (width < 300px) { .attributes { flex-direction: column; gap: 0; } .attr-split { display: none; } }
@media (height < 500px) { .doll { padding: var(--space-xxs) var(--space-xs); } .column { gap: var(--space-xxs); } [part="RpgCharacter.cell"] { width: var(--size-lg); height: var(--size-lg); padding: var(--space-xxs); } }
/* A short screen (a desktop at large text): smaller slots, so the doll can give height to the attributes. */
@media (600px <= width) and (500px <= height < 800px) { [part="RpgCharacter.cell"] { width: var(--size-lg); height: var(--size-lg); padding: var(--space-xxs); } .column { gap: var(--space-xxs); } }
@media (600px <= width) and (500px <= height < 800px) { .attr-mark { display: none; } .attributes { gap: var(--space-sm); } }
</style>
