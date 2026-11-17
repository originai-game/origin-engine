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
:host { align-items: start; max-inline-size: 100%; --rarity: var(--line-strong); }
[part="TooltipCard.frame"] { width: var(--measure-md); max-width: 100%; }
.common { --rarity: var(--common); }
.uncommon { --rarity: var(--uncommon); }
.rare { --rarity: var(--rare); }
.epic { --rarity: var(--epic); }
.legendary { --rarity: var(--legendary); }
.mythic { --rarity: var(--mythic); }
/* A rarity is the frame's rim and the title's ink, through their tokens, so a pack's frame still dresses it. */
#Card.common, #Card.uncommon, #Card.rare, #Card.epic, #Card.legendary, #Card.mythic { --frame-rim: var(--rarity); --frame-rim-width: var(--stroke-thin); --title-ink: var(--rarity); }
#Header { width: stretch; }
[part="TooltipCard.well"] { width: var(--size-touch); height: var(--size-touch); justify-content: center; overflow: hidden; }
#Art { width: var(--size-touch); height: var(--size-touch); }
.glyph { width: var(--size-sm); height: var(--size-sm); }
[part="TooltipCard.title"] { margin: 0; overflow-wrap: anywhere; }
.pip { --dot-fill: var(--rarity); --dot-radius: 0px; width: var(--space-xs); height: var(--space-xs); rotate: 45deg; }
#Subtitle { line-clamp: 2; }
.rule { width: stretch; }
.stats { width: stretch; margin: 0; padding: 0; list-style: none; }
[part="TooltipCard.row"] { width: stretch; }
[part="TooltipCard.label"] { overflow-wrap: anywhere; }
[part="TooltipCard.figure"] { white-space: nowrap; }
.mark { width: var(--space-sm); height: var(--space-sm); fill: currentColor; }
.down { rotate: 180deg; }
.amount { --figure-ink: currentColor; white-space: nowrap; }
[data-change="removed"] { --ink-label: var(--ink-muted); --figure-ink: var(--ink-muted); }
#Body, #Footer { margin: 0; }
#Hints { margin: 0; padding: 0; list-style: none; }
[part="TooltipCard.badge"] { white-space: nowrap; }
</style>
