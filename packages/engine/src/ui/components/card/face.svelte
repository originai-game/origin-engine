<meta name="component" content="CardFace">
<meta name="strings" content="./card.en.ogt">
<meta name="parts" content="CardFace.frame CardFace.cost CardFace.art CardFace.name CardFace.type CardFace.text">
<meta name="prop:name" content="string" data-doc="The card's name as the game words it (an upgraded card's name included).">
<meta name="prop:cost" content="string" data-doc="Its cost as the game words it (2, X, 0); empty hides the cost gem (a relic, an unplayable curse).">
<meta name="prop:icon" content="string" data-doc="Its art as an icon in the style's or the game's atlas, used when art is 0.">
<meta name="prop:art" content="number" data-doc="A painted picture (og.load.texture) filling the art window; 0 shows the icon.">
<meta name="prop:type" content="string" data-doc="The card's type for the pack's frame (attack, skill, power, curse, status, or the game's own word); it reaches the frame as data-type.">
<meta name="prop:typeLabel" content="string" data-doc="The type line as the game words it (Attack, Skill · Rare); empty hides it.">
<meta name="prop:rarity" content="string" data-doc="common, uncommon, rare, epic, legendary or mythic; it reaches the frame as data-rarity.">
<meta name="prop:text" content="string" data-doc="The rules text as the game words it, every number already computed; it wraps and is cut at the card's foot.">
<meta name="fixture:default" content='{"name":"Ember Strike","cost":"1","icon":"fire","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 6 damage. Apply 1 Burn."}'>
<meta name="fixture:relic" content='{"name":"Lantern of Dusk","cost":"","icon":"crown","art":0,"type":"relic","typeLabel":"Relic","rarity":"rare","text":"At the start of each combat, gain 1 energy."}'>
<meta name="fixture:overflow" content='{"name":"Thousandfold Lamentation of the Drowned Cathedral","cost":"10","icon":"skull","art":0,"type":"power","typeLabel":"Power · Legendary","rarity":"legendary","text":"At the start of your turn, every enemy loses 3 Strength, you draw 2 cards, gain 2 Block for each card in your discard pile and Exhaust a random card from your hand. Ethereal. Innate. Retain."}'>

<!--
  One card's face: the cost gem, the art window, the name plate, the type
  line and the rules text. It draws what the game words and reports
  nothing; the hand, the collection and the reward wrap it in their
  buttons. It fills the box it is given (give the box the card's aspect
  ratio); long names cut at one line and the rules text at the foot.
-->
<div id="CardFace" part="CardFace.frame" class="bare" data-type={type} data-rarity={rarity}>
  <div class="window"><img class="picture" src={art} alt="" hidden={!art}><og-icon part="CardFace.art" class="art" icon={icon} hidden={!!art}></og-icon></div>
  <p part="CardFace.name" class="name">{name}</p>
  <p part="CardFace.type" class="type" hidden={!typeLabel}>{typeLabel}</p>
  <p part="CardFace.text" class="text">{text}</p>
  <div part="CardFace.cost" class="cost" hidden={!cost} aria-label={t("card.face.cost", { cost })}><span class="cost-figure">{cost}</span></div>
</div>

<style>
[part="CardFace.frame"][hidden], [part="CardFace.frame"] [hidden] { display: none; }
[part="CardFace.frame"] { position: relative; display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-xxs); padding: var(--space-xs); overflow: hidden; background: var(--panel-raised); }
.window { position: relative; display: flex; width: stretch; height: 46%; flex: none; align-items: center; justify-content: center; overflow: hidden; }
.picture { width: stretch; height: stretch; object-fit: cover; }
.art { width: 62%; height: 62%; }
.name { margin: 0; width: stretch; flex: none; text-align: center; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.type { margin: 0; width: stretch; flex: none; text-align: center; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.text { margin: 0; width: stretch; flex: 1; min-height: 0; text-align: center; overflow: hidden; padding: 0 var(--space-xxs); }
.cost { position: absolute; left: 0; top: 0; display: flex; width: var(--size-md); height: var(--size-md); align-items: center; justify-content: center; }
.cost:dir(rtl) { left: auto; right: 0; }
.cost-figure { font-variant-numeric: tabular-nums; white-space: nowrap; }
</style>
