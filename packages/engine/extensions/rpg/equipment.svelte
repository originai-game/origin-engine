<meta name="component" content="RpgEquipment">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="Equipment.slot Equipment.portrait Equipment.frame">
<meta name="prop:name" content="string" data-doc="The character's name.">
<meta name="prop:subtitle" content="string" data-doc="The line under it (Level 7 Fighter · Human).">
<meta name="prop:portrait" content="number" data-doc="A texture for the portrait (og.load.texture); 0 draws the person icon.">
<meta name="prop:slots" content="list" data-doc="The equipment slots in the game's order, two to a row: each { id, label, icon, name, rarity, empty, locked }; empty draws the slot's label as its hint, locked one the game will not change.">
<meta name="prop:selected" content="string" data-doc="The id of the selected slot; empty is none.">
<meta name="prop:stats" content="list" data-doc="The character's numbers as the game computes them: each { id, label, value, delta, change }, delta the preview of the selected item (+6) and change better, worse or same.">
<meta name="action:details" content="Unequip">
<meta name="fixture:default" content='{"name":"Brenna Ashford","subtitle":"Level 7 Fighter · Human","portrait":0,"selected":"off","slots":[{"id":"head","label":"Head","icon":"crown","name":"Warden’s Helm","rarity":"uncommon","empty":false,"locked":false},{"id":"chest","label":"Chest","icon":"shield","name":"Brigandine","rarity":"rare","empty":false,"locked":false},{"id":"hands","label":"Hands","icon":"","name":"","rarity":"common","empty":true,"locked":false},{"id":"main","label":"Main hand","icon":"sword","name":"Emberwake Blade","rarity":"epic","empty":false,"locked":false},{"id":"off","label":"Off hand","icon":"","name":"","rarity":"common","empty":true,"locked":false},{"id":"ring","label":"Ring","icon":"gem","name":"Moonstone Band","rarity":"legendary","empty":false,"locked":true}],"stats":[{"id":"armour","label":"Armour class","value":"17","delta":"+2","change":"better"},{"id":"hp","label":"Hit points","value":"64","delta":"","change":"same"},{"id":"speed","label":"Speed","value":"9 m","delta":"−1.5 m","change":"worse"}]}'>

<!--
  The rpg kit's equipment (README.md): the character's portrait between
  two columns of slots, and the numbers the game computes for them, with
  the selected item's preview. The game decides what fits a slot and what
  a change does; the equipment reports choose (the slot's key) and
  unequip. On a narrow screen the slots wrap under the portrait.
-->
<section id="RpgEquipment" part="Equipment.frame" aria-label={name}>
  <div id="Doll">
    <div id="Slots" class="column" role="list">
      {#each slots as s (s.id)}
        <button id="Slot" part="Equipment.slot" class="slot" class:empty={s.empty} class:uncommon={s.rarity == "uncommon"} class:rare={s.rarity == "rare"} class:epic={s.rarity == "epic"} class:legendary={s.rarity == "legendary"} class:mythic={s.rarity == "mythic"} role="listitem" aria-selected={s.id == selected} aria-label={s.empty ? s.label : s.label + ": " + s.name} disabled={s.locked} onclick={choose}>
          <icon class="art" icon={s.icon} hidden={s.empty}></icon>
          <div class="ring"></div>
          <div class="rarity" hidden={s.empty || s.rarity == "common"}></div>
          <span class="hint" hidden={!s.empty}>{s.label}</span>
        </button>
      {/each}
    </div>
    <div id="Figure">
      <div id="Portrait" part="Equipment.portrait">
        <img id="Face" src={portrait} hidden={!portrait}>
        <icon id="Placeholder" icon="person" hidden={!!portrait}></icon>
      </div>
      <h2 id="Name">{name}</h2>
      <p id="Subtitle">{subtitle}</p>
    </div>
  </div>
  <div id="Numbers" role="list" aria-label={t("rpg.equipment.stats")}>
    {#each stats as s (s.id)}
      <div class="stat" role="listitem">
        <span class="what">{s.label}</span>
        <span class="value number">{s.value}</span>
        <span class="delta number" class:better={s.change == "better"} class:worse={s.change == "worse"} hidden={!s.delta}>{s.delta}</span>
      </div>
    {/each}
  </div>
  <div id="Actions">
    <button id="Unequip" variant="ghost" disabled={!selected} onclick={unequip} ondetails={unequip}><kbd id="UnequipKey" data-action="details"></kbd><span>{t("rpg.equipment.unequip")}</span></button>
  </div>
</section>

<style>
#RpgEquipment { width: stretch; gap: var(--space-md); padding: var(--space-lg); background: var(--panel); border-radius: var(--radius-lg); container-type: inline-size; }
#Doll { display: grid; grid-template-columns: auto 1fr; gap: var(--space-md); width: stretch; align-items: start; }
.column { display: grid; grid-template-columns: repeat(2, var(--size-xl)); gap: var(--space-xs); }
.slot { width: var(--size-xl); height: var(--size-xl); padding: var(--space-xs); align-items: center; justify-content: center; background: var(--well); border-radius: var(--radius-sm); }
.slot { position: relative; }
.slot.empty { opacity: 0.8; }
.ring { display: none; position: absolute; left: 0; top: 0; width: stretch; height: stretch; border: 2px solid var(--highlight); border-radius: var(--radius-sm); box-shadow: inset 0 0 10px 0 var(--highlight); }
.slot[aria-selected=true] .ring { display: block; }
.rarity { position: absolute; left: 22%; right: 22%; bottom: var(--space-xxs); height: 3px; border-radius: var(--radius-pill); }
.uncommon .rarity { background: var(--uncommon); }
.rare .rarity { background: var(--rare); }
.epic .rarity { background: var(--epic); }
.legendary .rarity { background: var(--legendary); }
.mythic .rarity { background: var(--mythic); }
.art { width: var(--size-lg); height: var(--size-lg); color: var(--ink); }
.hint { font: var(--text-caption); color: var(--ink-faint); text-align: center; width: stretch; }
#Figure { width: stretch; gap: var(--space-xs); align-items: center; }
#Portrait { width: stretch; max-width: calc(12 * var(--space-lg)); aspect-ratio: 1; align-items: center; justify-content: center; overflow: hidden; background: var(--panel-raised); border-radius: var(--radius-md); }
#Face { width: stretch; height: stretch; object-fit: cover; }
#Placeholder { width: var(--size-xl); height: var(--size-xl); color: var(--ink-faint); }
#Name { margin: 0; font: var(--text-xl); font-family: fantasy; color: var(--ink); text-align: center; width: stretch; }
#Subtitle { margin: 0; font: var(--text-sm); color: var(--ink-muted); text-align: center; width: stretch; }
#Numbers { display: grid; grid-template-columns: repeat(auto-fill, minmax(calc(14 * var(--space-md)), 1fr)); gap: var(--space-xxs) var(--space-lg); width: stretch; border-top: var(--stroke-hairline) solid var(--line); padding-top: var(--space-sm); }
.stat { display: grid; grid-template-columns: 1fr auto auto; gap: var(--space-sm); width: stretch; align-items: baseline; }
.what { color: var(--ink-muted); }
.value { color: var(--ink); }
.number { font-family: monospace; font-variant-numeric: tabular-nums; white-space: nowrap; text-align: end; }
.delta { min-width: calc(4 * var(--space-md)); color: var(--ink-muted); }
.delta.better { color: var(--success); }
.delta.worse { color: var(--danger); }
#Actions { display: flex; justify-content: end; width: stretch; }
#Actions button { min-height: var(--size-touch); }
@container (width < 480px) {
  #Doll { grid-template-columns: 1fr; }
  .column { grid-template-columns: repeat(auto-fill, minmax(var(--size-xl), 1fr)); width: stretch; order: 2; }
  .slot { width: stretch; aspect-ratio: 1; height: auto; }
}
@media (pointer: coarse) { kbd { display: none; } }
</style>
