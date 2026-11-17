<meta name="component" content="CardFace">
<meta name="strings" content="./card.en.ogt">
<meta name="parts" content="CardFace.frame CardFace.cost CardFace.art CardFace.name CardFace.type CardFace.text CardFace.picture CardFace.label CardFace.figure CardFace.well CardFace.headline CardFace.caption">
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
<div id="CardFace" part="CardFace.frame" class="bare" data-layout="stack" data-gap="tight" data-type={type} data-rarity={rarity}>
  <div part="CardFace.well" class="window" data-layout="row" data-gap="none">{#if art}<img class="picture" src={art} alt="">{:else}<og-icon part="CardFace.art CardFace.picture" class="art" icon={icon}></og-icon>{/if}</div>
  <p part="CardFace.name CardFace.headline" class="name">{name}</p>
  {#if typeLabel}<p part="CardFace.type CardFace.label CardFace.caption" class="type">{typeLabel}</p>{/if}
  <p part="CardFace.text CardFace.caption" class="text" data-fill>{text}</p>
  {#if cost}<div part="CardFace.cost CardFace.figure" class="cost" data-layout="row" data-gap="none" aria-label={t("card.face.cost", { cost })}><span class="cost-figure">{cost}</span></div>{/if}
</div>

<style>
:host { --headline-text: var(--text-sm); position: relative; inline-size: stretch; block-size: stretch; padding: var(--space-xs); overflow: hidden; }
.window { position: relative; height: 46%; flex: none; justify-content: center; overflow: hidden; }
.picture { position: absolute; inset: 0; inline-size: 100%; block-size: 100%; object-fit: cover; }
.art { width: 62%; height: 62%; }
.name { margin: 0; flex: none; text-align: center; line-clamp: 2; overflow: hidden; }
.type { margin: 0; flex: none; text-align: center; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.text { --caption-text: var(--text-sm); margin: 0; text-align: start; overflow: hidden; padding: var(--space-xxs); }
.cost { position: absolute; inset-inline-start: 0; top: 0; width: var(--size-md); height: var(--size-md); justify-content: center; }
.cost-figure { white-space: nowrap; }
</style>
