<meta name="component" content="CardHand">
<meta name="strings" content="./card.en.ogt">
<meta name="parts" content="CardHand.frame CardHand.slot CardHand.card CardHand.badge">
<meta name="prop:cards" content="list" data-doc="The hand in the game's order, keyed rows of { id, name, cost, icon, art, type, typeLabel, rarity, text, playable, at }: playable is the game's answer (cost, targets, rules), never the hand's; at is the card's place across a fanned hand from -1 (first) to 1 (last), 0 the middle, which the game derives from the order.">
<meta name="prop:selected" content="string" data-doc="The id of the card the player picked up (it rises and offers Play when playable); empty is none.">
<meta name="prop:fan" content="number" data-doc="How far a fanned hand turns its end cards, in degrees (the game's choice, 0 a flat row).">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the hand; empty is Hand.">
<meta name="action:details" content="Play">
<meta name="fixture:default" content='{"label":"","selected":"strike","fan":6,"cards":[{"id":"strike","name":"Ember Strike","cost":"1","icon":"fire","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 6 damage. Apply 1 Burn.","playable":true,"at":-1},{"id":"guard","name":"Iron Guard","cost":"1","icon":"shield","art":0,"type":"skill","typeLabel":"Skill","rarity":"common","text":"Gain 5 Block.","playable":true,"at":-0.33},{"id":"surge","name":"Arcane Surge","cost":"3","icon":"bolt","art":0,"type":"power","typeLabel":"Power","rarity":"rare","text":"Draw 2 cards each turn.","playable":false,"at":0.33},{"id":"wound","name":"Wound","cost":"","icon":"skull","art":0,"type":"status","typeLabel":"Status","rarity":"common","text":"Unplayable.","playable":false,"at":1}]}'>
<meta name="fixture:empty" content='{"label":"","selected":"","fan":6,"cards":[]}'>
<meta name="fixture:single" content='{"label":"","selected":"","fan":0,"cards":[{"id":"strike","name":"Ember Strike","cost":"1","icon":"fire","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 6 damage.","playable":true,"at":0}]}'>
<meta name="fixture:disabled" content='{"label":"","selected":"surge","fan":0,"cards":[{"id":"strike","name":"Ember Strike","cost":"1","icon":"fire","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 6 damage.","playable":false,"at":-1},{"id":"surge","name":"Arcane Surge","cost":"3","icon":"bolt","art":0,"type":"power","typeLabel":"Power","rarity":"rare","text":"Draw 2 cards each turn.","playable":false,"at":1}]}'>
<meta name="fixture:overflow" content='{"label":"","selected":"c6","fan":10,"cards":[{"id":"c1","name":"Ember Strike","cost":"1","icon":"fire","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 6 damage.","playable":true,"at":-1},{"id":"c2","name":"Iron Guard","cost":"1","icon":"shield","art":0,"type":"skill","typeLabel":"Skill","rarity":"common","text":"Gain 5 Block.","playable":true,"at":-0.8},{"id":"c3","name":"Arcane Surge","cost":"3","icon":"bolt","art":0,"type":"power","typeLabel":"Power","rarity":"rare","text":"Draw 2 cards each turn.","playable":false,"at":-0.6},{"id":"c4","name":"Thousandfold Lamentation of the Drowned Cathedral","cost":"10","icon":"skull","art":0,"type":"power","typeLabel":"Power","rarity":"legendary","text":"At the start of your turn every enemy loses 3 Strength and you draw 2 cards.","playable":false,"at":-0.4},{"id":"c5","name":"Quickstep","cost":"0","icon":"leaf","art":0,"type":"skill","typeLabel":"Skill","rarity":"uncommon","text":"Draw 1 card.","playable":true,"at":-0.2},{"id":"c6","name":"Cleave","cost":"1","icon":"sword","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 8 damage to all enemies.","playable":true,"at":0},{"id":"c7","name":"Mend","cost":"2","icon":"heart","art":0,"type":"skill","typeLabel":"Skill","rarity":"uncommon","text":"Heal 6.","playable":true,"at":0.2},{"id":"c8","name":"Wound","cost":"","icon":"skull","art":0,"type":"status","typeLabel":"Status","rarity":"common","text":"Unplayable.","playable":false,"at":0.4},{"id":"c9","name":"Bash","cost":"2","icon":"sword","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 8 damage. Apply 2 Vulnerable.","playable":true,"at":0.6},{"id":"c10","name":"Ward","cost":"1","icon":"shield","art":0,"type":"skill","typeLabel":"Skill","rarity":"common","text":"Gain 8 Block.","playable":true,"at":0.8},{"id":"c11","name":"Burn","cost":"","icon":"fire","art":0,"type":"status","typeLabel":"Status","rarity":"common","text":"Unplayable. Take 2 damage at the end of your turn.","playable":false,"at":1}]}'>

<!--
  The player's hand: cards side by side, overlapping as the hand grows so
  every card keeps a visible edge, turned along a fan when the game asks
  for one. The selected card rises; when the game says it is playable a
  Play plate sits on it. A card the game says cannot be played is not
  disabled (it can still be picked up and read) but carries the
  unplayable mark and data-playable=false, and never reports play.
  Reports select (a card's key: tap, Enter, pad A) and play (the Play
  plate, or X / pad Y on a focused playable card, its key). The game
  decides legality, cost and targets.
-->
<section id="CardHand" part="CardHand.frame" class="bare" aria-label={label || t("card.hand.name")} style:--fan={fan}>
  {#if cards.length}
    <div class="cards" role="listbox" aria-label={label || t("card.hand.name")}>
      {#each cards as c (c.id)}
        <div part="CardHand.slot" data-selected={c.id == selected} data-playable={c.playable} style:--at={c.at}>
          <div class="stack">
            {#if c.playable}
              <button part="CardHand.card" role="option" data-type={c.type} data-rarity={c.rarity} data-playable={c.playable} aria-selected={c.id == selected} aria-label={c.name} aria-description={c.text} onclick={select} ondetails={play}>
                <div class="face"><CardFace name={c.name} cost={c.cost} icon={c.icon} art={c.art} type={c.type} typeLabel={c.typeLabel} rarity={c.rarity} text={c.text} /></div>
              </button>
            {:else}
              <button part="CardHand.card" role="option" data-type={c.type} data-rarity={c.rarity} data-playable={c.playable} aria-selected={c.id == selected} aria-label={c.name} aria-description={t("card.hand.unplayable")} onclick={select}>
                <div class="face"><CardFace name={c.name} cost={c.cost} icon={c.icon} art={c.art} type={c.type} typeLabel={c.typeLabel} rarity={c.rarity} text={c.text} /></div>
                <div part="CardHand.badge" class="unplayable" aria-label={t("card.hand.unplayable")}><og-icon icon="lock"></og-icon></div>
              </button>
            {/if}
            <button class="play primary" hidden={!(c.id == selected && c.playable)} onclick={play}>{t("card.hand.play")}</button>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="empty" role="status"><p class="empty-text">{t("card.hand.empty")}</p></div>
  {/if}
</section>

<style>
[part="CardHand.frame"][hidden], [part="CardHand.frame"] [hidden] { display: none; }
[part="CardHand.frame"] { display: flex; width: stretch; height: stretch; align-items: flex-end; justify-content: center; container-type: size; }
/* A card is the frame's height less the room the picked-up card rises
   into and the room the fan's end cards dip into, and 0.7 of that wide
   (a card's proportion), but at most 0.36 of the frame's width, so a
   tall hand on a narrow screen keeps every card in reach. The row keeps
   room at its ends for the fan's outer cards to lean into. */
.cards { --card-width: min(calc((100cqh - var(--space-xl) * 2 - var(--space-md)) * 0.7), 36cqw); --card-height: calc(var(--card-width) / 0.7); display: flex; width: stretch; height: stretch; align-items: flex-end; justify-content: center; padding: 0 var(--space-xl) var(--space-xl); }
/* A slot is the card's share of the row: a card's width while there is
   room, less as the hand grows, so the next card overlaps it. The last
   slot keeps the whole card. */
[part="CardHand.slot"] { position: relative; height: var(--card-height); flex: 0 1 var(--card-width); min-width: 0; transform-origin: 50% 100%; rotate: calc(var(--at, 0) * var(--fan, 0) * 1deg); translate: 0 calc(var(--at, 0) * var(--at, 0) * var(--fan, 0) * var(--space-xxs)); transition: translate var(--motion-duration-fast) var(--motion-easing-out), rotate var(--motion-duration-fast) var(--motion-easing-out); }
[part="CardHand.slot"]:last-child { flex-shrink: 0; }
[part="CardHand.slot"]:hover { translate: 0 calc(var(--space-sm) * -1); }
[part="CardHand.slot"][data-selected=true] { rotate: 0deg; translate: 0 calc(var(--size-touch) * -1 - var(--space-xs)); z-index: 1; }
.stack { position: absolute; left: 0; bottom: 0; width: var(--card-width); height: stretch; }
.stack:dir(rtl) { left: auto; right: 0; }
[part="CardHand.card"] { position: relative; width: stretch; height: stretch; padding: 0; align-items: stretch; justify-content: stretch; }
[part="CardHand.card"] .face { width: stretch; height: stretch; }
/* Under the cost seal, on the edge the next card leaves uncovered. */
.unplayable { position: absolute; left: var(--space-xxs); top: calc(var(--size-md) + var(--space-xs)); display: flex; width: var(--size-sm); height: var(--size-sm); align-items: center; justify-content: center; }
.unplayable:dir(rtl) { left: auto; right: var(--space-xxs); }
.unplayable og-icon { width: 70%; height: 70%; }
/* Below the card's foot, in the room the lift leaves, so it never covers the rules text. */
.play { position: absolute; left: 10%; top: calc(100% + var(--space-xxs)); width: 80%; min-height: var(--size-touch); }
.empty { display: flex; width: stretch; height: stretch; align-items: center; justify-content: center; }
.empty-text { margin: 0; }
@media (prefers-reduced-motion: reduce) { [part="CardHand.slot"] { transition: none; } }
</style>
