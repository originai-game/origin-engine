<meta name="component" content="CardReward">
<meta name="strings" content="./card.en.ogt">
<meta name="prop:title" content="string" data-doc="The heading as the game words it (Choose a reward, Treasure).">
<meta name="prop:kicker" content="string" data-default="" data-doc="A small line over the title naming the moment (Victory · Floor 7); empty hides it.">
<meta name="prop:choices" content="list" data-doc="The offers the game rolled, keyed rows of { id, relic, name, cost, icon, art, type, typeLabel, rarity, text, enabled }: relic true draws a relic (its icon, typeLabel or Relic, name and text) instead of a card; enabled false when the game will not let it be taken.">
<meta name="prop:selected" content="string" data-doc="The offer the player is looking at (focused or tapped once); empty is none.">
<meta name="prop:skipLabel" content="string" data-doc="The skip button's text (Skip, Take nothing); empty hides it (Back still reports skip; a game that offers none ignores it).">
<meta name="action:back" content="Skip">
<meta name="fixture:default" content='{"title":"Choose a reward","kicker":"Victory · Floor 7","selected":"cleave","skipLabel":"Skip","choices":[{"id":"cleave","relic":false,"name":"Cleave","cost":"1","icon":"sword","art":0,"type":"attack","typeLabel":"Attack","rarity":"common","text":"Deal 8 damage to all enemies.","enabled":true},{"id":"mend","relic":false,"name":"Mend","cost":"2","icon":"heart","art":0,"type":"skill","typeLabel":"Skill","rarity":"uncommon","text":"Heal 6. Exhaust.","enabled":true},{"id":"surge","relic":false,"name":"Arcane Surge","cost":"3","icon":"bolt","art":0,"type":"power","typeLabel":"Power","rarity":"rare","text":"Draw 2 cards each turn.","enabled":true}]}'>
<meta name="fixture:relics" content='{"title":"Treasure","kicker":"","selected":"","skipLabel":"Take nothing","choices":[{"id":"lantern","relic":true,"name":"Lantern of Dusk","cost":"","icon":"crown","art":0,"type":"relic","typeLabel":"","rarity":"rare","text":"At the start of each combat, gain 1 energy.","enabled":true},{"id":"vial","relic":true,"name":"Bottled Storm","cost":"","icon":"drop","art":0,"type":"relic","typeLabel":"","rarity":"epic","text":"Your first attack each turn deals 4 more damage.","enabled":false}]}'>
<meta name="fixture:empty" content='{"title":"Choose a reward","kicker":"","selected":"","skipLabel":"Continue","choices":[]}'>

<!--
  Pick one of the game's offers: cards drawn as cards, relics as a gold
  plate with their mark, kind, name and text, and a skip button. The game
  rolls the offers and applies the pick; the screen reports pick (an
  offer's key: tap, Enter, pad A) and skip (the button, Escape, pad B).
  An offer the game refuses is disabled and dimmed; the one the player is
  looking at wears brackets. The offers share one row while it fits; a
  phone shows one at nearly full width, so its text stays whole, with the
  next one peeking at the faded edge, and scrolls sideways.
-->
<section id="CardReward" part="frame" variant="bare" layout="stack" fill gap="wide" aria-label={title} onback={skip}>
  <header class="band" part="band" layout="stack" gap="hair">{#if kicker}<span part="kicker" class="kicker">{kicker}</span>{/if}<h2 class="title" part="title">{title}</h2></header>
  <div class="strip" fill>
    <div class="choices" layout="row" gap="loose" role="listbox" aria-label={t("card.reward.choices")}>
      {#each choices as c (c.id)}
        {#if c.relic}
          <button class="relic" part="relic" role="option" tier={c.rarity} disabled={!c.enabled} aria-selected={c.id == selected} aria-label={c.name} aria-description={c.text} onclick={pick}><div class="offer">
            <div class="relic-face" layout="stack" gap="tight">
              <div part="mark" class="mark" layout="row" gap="none"><og-icon class="mark-icon" icon={c.icon}></og-icon></div>
              <span part="kind" class="kind">{c.typeLabel || t("card.reward.relic")}</span>
              <span part="name" class="name">{c.name}</span>
              <span part="text" class="text">{c.text}</span>
            </div>
            {#if c.id == selected}<div part="bracket" class="bracket"></div>{/if}
          </div></button>
        {:else}
          <button class="choice" part="choice" role="option" data-type={c.type} tier={c.rarity} disabled={!c.enabled} aria-selected={c.id == selected} aria-label={c.name} aria-description={c.text} onclick={pick}><div class="offer">
            <div class="face"><CardFace name={c.name} cost={c.cost} icon={c.icon} art={c.art} type={c.type} typeLabel={c.typeLabel} rarity={c.rarity} text={c.text} dim={!c.enabled} /></div>
            {#if c.id == selected}<div part="bracket" class="bracket"></div>{/if}
          </div></button>
        {/if}
      {:else}
        <p class="empty-text" role="status">{t("card.reward.empty")}</p>
      {/each}
    </div>
  </div>
  {#if skipLabel}<footer class="foot" layout="row">
    <button id="Skip" onclick={skip}>{skipLabel}</button>
  </footer>{/if}
</section>

<style>
#CardReward { align-items: center; justify-content: center; container-type: size; }
.band { width: stretch; align-items: center; flex: none; }
.kicker { text-align: center; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-size: var(--inv-kicker-size); font-weight: var(--inv-name-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--tcg-label-ink); }
.title { margin: 0; text-align: center; }
/* The offers share one row, each at most three icon sizes wide and no
   taller than the height left under the band and over Skip. */
.strip { width: stretch; min-height: 0; }
.choices { width: stretch; height: stretch; align-items: center; justify-content: center; padding: var(--space-sm); }
.choice, .relic { --offer-width: min(calc(var(--size-xxl) * 3), calc((100cqh - var(--size-lg) * 4) * 0.7)); flex: 0 1 var(--offer-width); min-width: calc(var(--size-xxl) * 1.1); aspect-ratio: 0.7; padding: 0; align-items: stretch; justify-content: stretch; border: none; border-radius: 0; background: var(--inv-bare-fill); box-shadow: none; }
.offer { position: relative; width: stretch; height: stretch; }
.face { width: stretch; height: stretch; }
.bracket { position: absolute; inset: calc(var(--space-xxs) * -1); background: var(--inv-bracket); filter: var(--inv-bracket-glow); pointer-events: none; }
/* A relic: gold-rimmed glass, its mark in a diamond over a gold glow. */
.relic-face { width: stretch; height: stretch; align-items: center; justify-content: center; padding: var(--space-sm); overflow: hidden; text-align: center; background: var(--tcg-relic-fill); box-shadow: var(--tcg-relic-rim); border-radius: var(--tcg-face-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.relic:disabled .relic-face { filter: var(--tcg-dim); }
.mark { width: 42%; aspect-ratio: 1; flex: none; justify-content: center; align-items: center; clip-path: polygon(50% 0, 100% 50%, 50% 100%, 0 50%); background: var(--tcg-relic-mark-fill); }
.mark-icon { width: 58%; height: 58%; color: var(--tcg-relic); }
.kind { flex: none; width: stretch; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-size: var(--tcg-type-size); font-weight: var(--tcg-type-weight); letter-spacing: var(--tcg-type-tracking); text-transform: var(--tcg-type-transform); color: var(--tcg-relic); }
.name { flex: none; width: stretch; line-clamp: 2; overflow: hidden; overflow-wrap: anywhere; font-size: var(--tcg-name-size); font-weight: var(--tcg-name-weight); color: var(--ink-bright); }
.text { width: stretch; min-height: 0; line-clamp: 5; overflow: hidden; overflow-wrap: anywhere; font-size: var(--tcg-text-size); color: var(--tcg-text-ink); }
.empty-text { margin: 0; text-align: center; }
.foot { flex: none; justify-content: center; }
#Skip { min-width: calc(var(--size-xxl) * 2); min-height: var(--size-touch); }
/* A phone: one offer at nearly the full width, so its text stays whole,
   the next one peeking at the faded edge; the row scrolls sideways. */
@container (width < 640px) {
  .strip { overflow-x: auto; overflow-y: hidden; scrollbar-width: none; mask-image: var(--tcg-fade); }
  .choices { width: auto; justify-content: start; scroll-snap-type: x mandatory; padding-inline-end: var(--size-lg); }
  .choice, .relic { flex: 0 0 min(78cqw, calc((100cqh - var(--size-lg) * 4) * 0.7)); min-width: 0; scroll-snap-align: center; }
}
</style>
