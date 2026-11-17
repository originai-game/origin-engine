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
[part="RpgCharacter.band"] { width: stretch; justify-content: center; flex: none; }
.band-end { width: var(--size-lg); height: var(--space-lg); flex: none; }
.who { width: stretch; flex: none; }
.portrait { width: calc(var(--size-xxl) * 1.15); height: calc(var(--size-xxl) * 1.15); object-fit: cover; flex: none; }
.who-text { align-items: center; text-align: center; }
[part="RpgCharacter.name"], .char-line { margin: 0; }
.doll { position: relative; width: stretch; justify-content: space-between; padding: var(--space-sm) var(--space-xs); }
.figure { position: absolute; inset-inline-start: 15%; top: 0; bottom: 0; inline-size: 70%; block-size: 100%; object-fit: contain; object-position: center bottom; }
.column { height: stretch; min-height: 0; justify-content: space-between; }
[part="RpgCharacter.cell"] { flex: none; width: var(--size-xl); height: var(--size-xl); padding: var(--space-xs); }
[part="RpgCharacter.cell"] .face { width: stretch; height: stretch; }
.art { width: stretch; height: stretch; }
.attributes { flex: none; padding: var(--space-sm) var(--space-xs) 0; --layout-switch-at: var(--measure-xs); }
.attr-column { align-self: stretch; }
.attr-split { width: var(--stroke-hairline); align-self: stretch; margin: 0; }
[part="RpgCharacter.row"] { width: stretch; padding: var(--space-xs) 0; }
.attr-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
.attr-label { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.attr-value { white-space: nowrap; }
@media (width < 600px) { [part="RpgCharacter.cell"] { width: var(--size-touch); height: var(--size-touch); } }
@container (width < 300px) { .attr-split { display: none; } }
@media (height < 500px) { .doll { padding: var(--space-xxs) var(--space-xs); } .column { --layout-gap-tight: var(--layout-gap-hair); } [part="RpgCharacter.cell"] { width: var(--size-lg); height: var(--size-lg); padding: var(--space-xxs); } }
/* A short screen (a desktop at large text): smaller slots, so the doll can give height to the attributes. */
@media (600px <= width) and (500px <= height < 800px) { [part="RpgCharacter.cell"] { width: var(--size-lg); height: var(--size-lg); padding: var(--space-xxs); } .column { --layout-gap-tight: var(--layout-gap-hair); } }
@media (600px <= width) and (500px <= height < 800px) { .attr-mark { display: none; } .attributes { --layout-gap-loose: var(--layout-gap); } }
</style>
