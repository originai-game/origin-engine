<meta name="component" content="TooltipCard">
<meta name="strings" content="./tooltip-card.en.ogt">
<meta name="parts" content="TooltipCard.frame TooltipCard.title TooltipCard.label TooltipCard.row TooltipCard.figure TooltipCard.well TooltipCard.badge TooltipCard.change TooltipCard.delta TooltipCard.mark TooltipCard.caption TooltipCard.dot">
<meta name="prop:title" content="string" data-doc="The title, in its rarity's colour.">
<meta name="prop:subtitle" content="string" data-doc="A line under it (Legendary sword, Passive); empty: none.">
<meta name="prop:rarity" content="string" data-doc="none, common, uncommon, rare, epic, legendary or mythic: the title's and frame's colour and its pips (one to six, so no tier is told by colour alone).">
<meta name="prop:icon" content="string" data-doc="An icon beside the title when there is no image; empty: none.">
<meta name="prop:image" content="number" data-doc="An art slot beside the title (an item or ability texture handle, 1:1); 0: the icon.">
<meta name="prop:body" content="string" data-doc="The body text; empty: none.">
<meta name="prop:stats" content="list" data-doc="Keyed { id, label, value, delta, deltaText, better, change? } comparison facts supplied by the game. Signed delta and its arrow describe the numeric direction; better true means lower is better, flipping the colours. Optional change is added or removed; absent means both items have the stat. Added shows value without delta; removed shows the compared value muted. Presence is translated and exposed as data-change, never inferred from strings.">
<meta name="prop:compareTitle" content="string" data-doc="The heading over a second card of what is equipped (Equipped); empty draws no second card.">
<meta name="prop:compareStats" content="list" data-doc="The equipped item's stat rows, { id, label, value }.">
<meta name="prop:footer" content="string" data-doc="A last line (Sell for 120, a flavour quote) in the muted ink; empty: none.">
<meta name="prop:hints" content="list" data-doc="Action hints (Right-click to equip), a keyed collection of { id, text, action }: action an input action whose glyph leads the hint; empty: none.">
<meta name="fixture:default" content='{"title":"Ember Blade","subtitle":"Legendary sword","rarity":"legendary","icon":"sword","image":0,"body":"Burns on hit for 12 fire damage over 3 seconds. Two-handed.","stats":[{"id":"dmg","label":"Damage","value":"42–58","delta":12,"deltaText":"+12","better":false},{"id":"speed","label":"Attack speed","value":"1.1","delta":-0.2,"deltaText":"−0.2","better":false},{"id":"cd","label":"Cooldown","value":"4 s","delta":0,"deltaText":"","better":true}],"compareTitle":"Equipped","compareStats":[{"id":"dmg","label":"Damage","value":"30–46"},{"id":"speed","label":"Attack speed","value":"1.3"},{"id":"cd","label":"Cooldown","value":"4 s"}],"footer":"Sell for 120","hints":[{"id":"equip","text":"Equip","action":"confirm"}]}'>

<meta name="fixture:comparison" content='{"title":"The Ember Blade of the Last Watcher of the Ninefold Gate","stats":[{"id":"damage","label":"Damage against expedition guardians","value":"58","delta":12,"deltaText":"+12","better":false},{"id":"cooldown","label":"Recovery time after a charged attack","value":"3 s","delta":-1,"deltaText":"−1 s","better":true},{"id":"fire","label":"Resistance to burning ground in the northern territories","value":"8%","change":"added"},{"id":"poison","label":"Resistance to poison from creatures of the Sunken Kingdom","value":"12%","change":"removed"}],"compareTitle":"Equipped","compareStats":[{"id":"damage","label":"Damage against expedition guardians","value":"46"},{"id":"cooldown","label":"Recovery time after a charged attack","value":"4 s"},{"id":"poison","label":"Resistance to poison from creatures of the Sunken Kingdom","value":"12%"}]}'>

<div id="TooltipCard" data-layout="cluster" role="tooltip" aria-label={title} aria-description={body}>
  {#if compareTitle != ""}
    <section id="Compare" part="TooltipCard.frame" data-layout="stack" class="card compare">
      <span id="Eyebrow" part="TooltipCard.caption" class="eyebrow">{compareTitle}</span>
      <hr class="rule">
      <ul class="stats" data-layout="stack" data-gap="hair">
        {#each compareStats as s (s.id)}
          <li class="stat" part="TooltipCard.row" data-layout="cluster">
            <span class="label" part="TooltipCard.label" data-fill>{s.label}</span>
            <span class="value" part="TooltipCard.figure">{s.value}</span>
          </li>
        {/each}
      </ul>
    </section>
  {/if}
  <section id="Card" part="TooltipCard.frame" data-layout="stack" class="card" class:common={rarity == "common"} class:uncommon={rarity == "uncommon"} class:rare={rarity == "rare"} class:epic={rarity == "epic"} class:legendary={rarity == "legendary"} class:mythic={rarity == "mythic"}>
    <header id="Header" data-layout="row">
      {#if image || icon}<div id="Well" part="TooltipCard.well" class="well" data-layout="row">
        {#if image}<img id="Art" src={image} alt="">
        {:else}<og-icon id="Icon" class="glyph" icon={icon}></og-icon>{/if}
      </div>{/if}
      <div id="Names" data-layout="stack" data-gap="hair" data-fill>
        <h3 id="Title" part="TooltipCard.title">{title}</h3>
        {#if subtitle}<div id="Sub" data-layout="cluster" data-gap="tight">
          <div id="Pips" data-layout="row" data-gap="tight">
            {#if rarity != 'none' && rarity != ''}<span part="TooltipCard.dot" class="pip"></span>{/if}
            {#if rarity != 'none' && rarity != '' && rarity != 'common'}<span part="TooltipCard.dot" class="pip"></span>{/if}
            {#if rarity == 'rare' || rarity == 'epic' || rarity == 'legendary' || rarity == 'mythic'}<span part="TooltipCard.dot" class="pip"></span>{/if}
            {#if rarity == 'epic' || rarity == 'legendary' || rarity == 'mythic'}<span part="TooltipCard.dot" class="pip"></span>{/if}
            {#if rarity == 'legendary' || rarity == 'mythic'}<span part="TooltipCard.dot" class="pip"></span>{/if}
            {#if rarity == 'mythic'}<span part="TooltipCard.dot" class="pip"></span>{/if}
          </div>
          <span id="Subtitle" part="TooltipCard.caption" class="subtitle">{subtitle}</span>
        </div>{/if}
      </div>
    </header>
    <hr class="rule">
    {#if stats.length}<ul class="stats" data-layout="stack" data-gap="hair">
      {#each stats as s (s.id)}
        <li class="stat" part="TooltipCard.row" data-layout="cluster" data-change={s.change}>
          <span class="label" part="TooltipCard.label" data-fill>{s.label}</span>
          <span class="value" part="TooltipCard.figure">{s.value}</span>
          {#if s.change == 'added' || s.change == 'removed'}<span part="TooltipCard.change TooltipCard.caption" class="change">{s.change == 'added' ? t("tooltip.change.added") : t("tooltip.change.removed")}</span>
          {:else if s.delta != 0}<div part="TooltipCard.delta" class="delta" data-layout="row" data-gap="hair" data-trend={(s.better ? s.delta < 0 : s.delta > 0) ? 'up' : 'down'} class:good={s.better ? s.delta < 0 : s.delta > 0} class:bad={s.better ? s.delta > 0 : s.delta < 0}>
            <svg part="TooltipCard.mark" class="mark" class:down={s.delta < 0} viewBox="0 0 8 7"><polygon points="4,0 8,7 0,7"></polygon></svg>
            <span part="TooltipCard.figure" class="amount">{s.deltaText}</span>
          </div>{/if}
        </li>
      {/each}
    </ul>{/if}
    {#if body}<p id="Body">{body}</p>{/if}
    {#if footer}<p id="Footer" part="TooltipCard.caption">{footer}</p>{/if}
    {#if hints.length}<ul id="Hints" data-layout="cluster">
      {#each hints as h (h.id)}
        <li class="hint" part="TooltipCard.badge" data-layout="row" data-gap="tight">{#if h.action}<kbd class="key" data-action={h.action}></kbd>{/if}<span>{h.text}</span></li>
      {/each}
    </ul>{/if}
  </section>
</div>

<style>
/* The item's card is a dense smoked plate lit along its top in its
   rarity's colour, its icon in an octagon socket of that colour; what is
   equipped stands beside it on a plainer plate, and under it on a narrow
   screen. Figures are condensed and tabular; a change carries a triangle
   and its sign in the better green or the worse red, an added or removed
   stat a chip. A long name or stat wraps between its words. */
:host { align-items: start; max-inline-size: 100%; --rarity: var(--line-strong); color: var(--front-ink); }
.common { --rarity: var(--common); }
.uncommon { --rarity: var(--uncommon); }
.rare { --rarity: var(--rare); }
.epic { --rarity: var(--epic); }
.legendary { --rarity: var(--legendary); }
.mythic { --rarity: var(--mythic); }
/* The frame, the socket, the title and the pips are their roles' parts,
   dressed through the roles' tokens, which the card sets from its rarity. */
[part="TooltipCard.frame"] { width: calc(300 * var(--font-step-px)); max-width: 100%; row-gap: var(--space-sm); --frame-pad: var(--space-md) var(--space-lg); --frame-radius: 0; --frame-rim-width: 0; --frame-fill: linear-gradient(180deg, color-mix(in srgb, var(--rarity) var(--dialog-tip-tint), transparent), transparent calc(90 * var(--font-step-px))), var(--dialog-fill); --frame-shadow: var(--dialog-tip-rim), inset 0 calc(2 * var(--font-step-px)) 0 var(--rarity), var(--dialog-lift); --title-ink: var(--rarity); --title-size: var(--dialog-tip-title-size); --title-tracking: 0; --ink: var(--front-ink); }
#Card:not(.common, .uncommon, .rare, .epic, .legendary, .mythic) { --title-ink: var(--front-ink); }
#Compare { --frame-fill: var(--hud-plate-fill); --frame-shadow: var(--dialog-tip-plain-edge); }
#Header { width: stretch; column-gap: var(--space-md); align-items: center; }
/* The socket: an octagon in the rarity's colour round a dark well. */
[part="TooltipCard.well"] { flex: none; width: calc(48 * var(--font-step-px)); height: calc(48 * var(--font-step-px)); justify-content: center; overflow: hidden; --well-radius: 0; --well-fill: linear-gradient(180deg, color-mix(in srgb, var(--rarity) 40%, transparent), transparent), var(--dialog-tip-well); --well-shadow: inset 0 0 0 calc(2 * var(--font-step-px)) var(--rarity); clip-path: var(--hud-octagon); }
#Art { width: 100%; height: 100%; }
.glyph { width: 60%; height: 60%; color: var(--front-ink); }
[part="TooltipCard.title"] { margin: 0; line-height: var(--font-leading-snug); overflow-wrap: break-word; }
.pip { flex: none; width: calc(6 * var(--font-step-px)); height: calc(6 * var(--font-step-px)); rotate: 45deg; --dot-fill: var(--rarity); --dot-radius: 0; --dot-rim-width: 0; }
#Sub { align-items: center; gap: var(--space-xxs) var(--space-sm); margin-block-start: var(--space-xxs); }
#Pips { gap: var(--space-xs); }
#Subtitle, .eyebrow { font-size: var(--hud-label-size); font-weight: var(--dialog-weight); line-height: var(--font-leading-snug); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--front-ink-dim); }
.eyebrow { color: var(--front-ink-faint); }
#Subtitle:lang(zh, ja, ko), .eyebrow:lang(zh, ja, ko), .change:lang(zh, ja, ko) { font-size: var(--dialog-heading-cjk-size); letter-spacing: 0; }
.rule { width: stretch; margin: 0; color: var(--dialog-rule); }
.stats { width: stretch; margin: 0; padding: 0; list-style: none; row-gap: 0; }
[part="TooltipCard.row"] { width: stretch; align-items: baseline; justify-content: end; gap: var(--space-xxs) var(--space-sm); padding: var(--space-xs) 0; border-radius: 0; background: none; border-block-end: var(--data-rule-line); }
[part="TooltipCard.row"]:last-child { border-block-end: 0; }
[part="TooltipCard.label"] { flex: 1 1 calc(110 * var(--font-step-px)); min-inline-size: 0; font-size: var(--dialog-tip-label-size); line-height: var(--font-leading-snug); letter-spacing: 0; text-transform: none; color: var(--front-ink-dim); overflow-wrap: break-word; }
.value { font-size: var(--dialog-tip-figure-size); font-weight: var(--dialog-strong-weight); font-stretch: var(--front-stretch); white-space: nowrap; color: var(--front-ink); }
[part="TooltipCard.delta"] { align-items: center; min-inline-size: calc(52 * var(--font-step-px)); justify-content: end; font-size: var(--dialog-tip-label-size); font-weight: var(--dialog-strong-weight); }
.good { color: var(--trend-up); }
.bad { color: var(--trend-down); }
.mark { flex: none; width: calc(9 * var(--font-step-px)); height: calc(8 * var(--font-step-px)); fill: currentColor; }
.down { rotate: 180deg; }
.amount { color: inherit; white-space: nowrap; }
[part~="TooltipCard.change"] { flex: none; padding: var(--space-xxs) var(--space-xs); font-size: var(--dialog-tip-chip-size); font-weight: var(--dialog-strong-weight); line-height: var(--font-leading-snug); letter-spacing: var(--hud-label-tracking); white-space: nowrap; color: var(--trend-up); background: var(--hud-tag-fill); box-shadow: var(--dialog-tip-chip-rim); }
[data-change="removed"] [part="TooltipCard.label"], [data-change="removed"] .value, [data-change="removed"] [part~="TooltipCard.change"] { color: var(--front-ink-faint); }
#Body, #Footer { margin: 0; }
#Body { font-size: var(--dialog-tip-label-size); line-height: var(--font-leading-normal); color: var(--front-ink-dim); }
#Footer { font-size: var(--dialog-tip-note-size); line-height: var(--font-leading-snug); color: var(--front-ink-faint); }
#Hints { width: stretch; margin: 0; padding: var(--space-sm) 0 0; list-style: none; border-block-start: var(--dialog-rule-line); gap: var(--space-xs) var(--space-md); }
[part="TooltipCard.badge"] { align-items: center; padding: 0; border-radius: 0; background: none; box-shadow: none; font-size: var(--dialog-tip-note-size); font-weight: var(--dialog-weight); white-space: nowrap; color: var(--front-ink); }
/* A narrow screen: the item's card first, what is equipped under it, each
   as wide as the screen allows. */
@media (width < 600px) {
  :host { flex-direction: column-reverse; flex-wrap: nowrap; align-items: stretch; }
  [part="TooltipCard.frame"] { width: auto; }
}
</style>
