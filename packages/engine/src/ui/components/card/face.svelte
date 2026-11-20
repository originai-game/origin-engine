<meta name="component" content="CardFace">
<meta name="strings" content="./card.en.ogt">
<meta name="prop:name" content="string" data-doc="The card's name as the game words it (an upgraded card's name included).">
<meta name="prop:cost" content="string" data-doc="Its cost as the game words it (2, X, 0); empty hides the cost gem (a relic, an unplayable curse).">
<meta name="prop:icon" content="string" data-doc="Its art as an icon in the style's or the game's atlas, used when art is 0.">
<meta name="prop:art" content="number" data-doc="A painted picture (og.load.texture) filling the art window; 0 shows the icon.">
<meta name="prop:type" content="string" data-doc="The card's type for the pack's frame (attack, skill, power, curse, status, or the game's own word); it reaches the frame as data-type.">
<meta name="prop:typeLabel" content="string" data-doc="The type line as the game words it (Attack, Skill · Rare); empty hides it.">
<meta name="prop:rarity" content="string" data-doc="common, uncommon, rare, epic, legendary or mythic; it reaches the frame as data-rarity.">
<meta name="prop:text" content="string" data-doc="The rules text as the game words it, every number already computed; it wraps, up to four lines and then an ellipsis, and the art window gives it the room.">
<meta name="prop:dim" content="bool" data-doc="Dims the art and the frame (a card that cannot be played or is not owned); the words keep their ink.">
<meta name="prop:size" content="string" data-doc="How short its place is: empty is full height; medium, small and tiny clamp the rules text to 3, 2 and 1 lines (tiny also drops the type line).">
<meta name="fixture:default" content='{"name":"Ember Strike","cost":"1","icon":"fire","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 6 damage. Apply 1 Burn.","dim":false}'>
<meta name="fixture:relic" content='{"name":"Lantern of Dusk","cost":"","icon":"crown","art":0,"type":"relic","typeLabel":"Relic","rarity":"rare","text":"At the start of each combat, gain 1 energy.","dim":false}'>
<meta name="fixture:overflow" content='{"name":"Thousandfold Lamentation of the Drowned Cathedral","cost":"10","icon":"skull","art":0,"type":"power","typeLabel":"Power · Legendary","rarity":"legendary","text":"At the start of your turn, every enemy loses 3 Strength, you draw 2 cards, gain 2 Block for each card in your discard pile and Exhaust a random card from your hand. Ethereal. Innate. Retain.","dim":false}'>
<meta name="fixture:dim" content='{"name":"Arcane Surge","cost":"3","icon":"bolt","art":0,"type":"power","typeLabel":"Power · Rare","rarity":"rare","text":"Draw 2 more cards each turn.","dim":true}'>

<!--
  One card's face: the cost gem, the art window over its rarity's glow,
  the name on its plate, the type line and the rules text. It draws what
  the game words and reports nothing; the hand, the collection and the
  reward wrap it in their buttons. It fills the box it is given (give the
  box the card's aspect ratio). The frame wears its type's colour; the
  name takes two lines and the rules text four, each ending in an
  ellipsis at a whole line, and the art window yields its height to them.
-->
<div class="frame" id="CardFace" part="frame" variant="bare" layout="stack" gap="hair" tier={rarity} class:type-attack={(type) == 'attack'} class:type-curse={(type) == 'curse'} class:type-power={(type) == 'power'} class:type-relic={(type) == 'relic'} class:type-skill={(type) == 'skill'} class:type-status={(type) == 'status'} class:dim={dim} class:medium={size == 'medium' || size == 'small' || size == 'tiny'} class:small={size == 'small' || size == 'tiny'} class:tiny={size == 'tiny'}>
  <div class="wash"></div>
  <div part="art" class="window" layout="row" gap="none"><div class="glow"></div>{#if art}<img class="picture" src={art} alt="">{:else}<og-icon class="icon" icon={icon}></og-icon>{/if}</div>
  <p part="name" class="name">{name}</p>
  {#if typeLabel}<p part="type" class="type">{typeLabel}</p>{/if}
  {#if text}<p part="text" class="text">{text}</p>{/if}
  {#if cost}<div part="cost" class="cost" layout="row" gap="none" aria-label={t("card.face.cost", { cost })}><span class="cost-figure">{cost}</span></div>{/if}
</div>

<style>
/* The frame: cut at two corners, rimmed and washed in its type's colour. */
#CardFace { container-type: size; position: relative; inline-size: stretch; block-size: stretch; padding: var(--space-xs); overflow: hidden; border-radius: var(--tcg-face-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; background: var(--tcg-face-fill); box-shadow: var(--tcg-face-rim); color: var(--tcg-neutral); }
.frame.type-attack { color: var(--tcg-attack); }
.frame.type-skill { color: var(--tcg-skill); }
.frame.type-power { color: var(--tcg-power); }
.frame.type-curse, .frame.type-status { color: var(--tcg-curse); }
.frame.type-relic { color: var(--tcg-relic); }
.wash { position: absolute; inset: 0; background: var(--tcg-face-wash); opacity: 0.22; }
/* The art window takes what the words leave, over its rarity's glow. */
.window { position: relative; flex: 1 1 0; min-block-size: 18%; justify-content: center; align-items: center; overflow: hidden; border-radius: var(--tcg-window-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; background: var(--tcg-window-fill); color: var(--common); }
.rarity-uncommon .window { color: var(--uncommon); }
.rarity-rare .window { color: var(--rare); }
.rarity-epic .window { color: var(--epic); }
.rarity-legendary .window { color: var(--legendary); }
.rarity-mythic .window { color: var(--mythic); }
.glow { position: absolute; inset: 0; background: var(--tcg-window-glow); opacity: 0.45; }
.picture { position: absolute; inset: 0; inline-size: 100%; block-size: 100%; object-fit: cover; }
.icon { position: relative; inline-size: 56%; block-size: 76%; color: var(--ink-bright); filter: var(--tcg-art-shadow); }
.dim .window, .dim .wash { filter: var(--tcg-dim); }
.frame.dim { color: var(--tcg-dim-rim); }
.name { position: relative; margin: 0; flex: none; padding: var(--space-xxs) var(--space-xs); text-align: center; overflow-wrap: anywhere; line-clamp: 2; overflow: hidden; font-size: var(--tcg-name-size); font-weight: var(--tcg-name-weight); line-height: 1.15; color: var(--ink-bright); background: var(--tcg-plate-fill); }
.type { position: relative; margin: 0; flex: none; text-align: center; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; font-size: var(--tcg-type-size); font-weight: var(--tcg-type-weight); letter-spacing: var(--tcg-type-tracking); text-transform: var(--tcg-type-transform); color: var(--tcg-neutral-ink); }
.type-attack .type { color: var(--tcg-attack-ink); }
.type-skill .type { color: var(--tcg-skill-ink); }
.type-power .type { color: var(--tcg-power-ink); }
.type-curse .type, .type-status .type { color: var(--tcg-curse-ink); }
.text { position: relative; margin: 0; flex: none; padding: var(--space-xxs) var(--space-xs); text-align: center; overflow-wrap: anywhere; line-clamp: 4; overflow: hidden; font-size: var(--tcg-text-size); line-height: 1.3; color: var(--tcg-text-ink); background: var(--tcg-text-fill); }
/* The cost: a hexagon gem over the top corner. */
.cost { position: absolute; inset-inline-start: 0; top: 0; inline-size: var(--tcg-gem-size); block-size: calc(var(--tcg-gem-size) * 1.12); justify-content: center; align-items: center; clip-path: polygon(50% 0, 100% 25%, 100% 75%, 50% 100%, 0 75%, 0 25%); background: var(--tcg-gem-fill); color: var(--tcg-gem-ink); font-size: var(--tcg-gem-figure-size); font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); }
.cost-figure { white-space: nowrap; }
/* A short face (a small cell, large text) gives its words fewer lines,
   each still ending whole with an ellipsis, so the frame never cuts one. */
#CardFace.medium .text { line-clamp: 3; }
#CardFace.medium .name { line-clamp: 1; }
#CardFace.small .text { line-clamp: 2; }
#CardFace.tiny .text { line-clamp: 1; }
#CardFace.tiny .type { display: none; }
</style>
