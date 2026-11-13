<meta name="component" content="TooltipCard">
<meta name="parts" content="TooltipCard.frame TooltipCard.title TooltipCard.label TooltipCard.row TooltipCard.figure TooltipCard.well TooltipCard.badge">
<meta name="prop:title" content="string" data-doc="The title, in its rarity's colour.">
<meta name="prop:subtitle" content="string" data-doc="A line under it (Legendary sword, Passive); empty: none.">
<meta name="prop:rarity" content="string" data-doc="none, common, uncommon, rare, epic, legendary or mythic: the title's and frame's colour and its pips (one to six, so no tier is told by colour alone).">
<meta name="prop:icon" content="string" data-doc="An icon beside the title when there is no image; empty: none.">
<meta name="prop:image" content="number" data-doc="An art slot beside the title (an item or ability texture handle, 1:1); 0: the icon.">
<meta name="prop:body" content="string" data-doc="The body text; empty: none.">
<meta name="prop:stats" content="list" data-doc="Stat rows, a keyed collection of { id, label, value, delta, deltaText, better }: delta against what is compared, positive better (an up mark in success), negative a down mark in danger, 0 none; deltaText as shown; better true when higher is worse (a cooldown), flipping the colours.">
<meta name="prop:compareTitle" content="string" data-doc="The heading over a second card of what is equipped (Equipped); empty draws no second card.">
<meta name="prop:compareStats" content="list" data-doc="The equipped item's stat rows, { id, label, value }.">
<meta name="prop:footer" content="string" data-doc="A last line (Sell for 120, a flavour quote) in the muted ink; empty: none.">
<meta name="prop:hints" content="list" data-doc="Action hints (Right-click to equip), a keyed collection of { id, text, action }: action an input action whose glyph leads the hint; empty: none.">
<meta name="fixture:default" content='{"title":"Ember Blade","subtitle":"Legendary sword","rarity":"legendary","icon":"sword","image":0,"body":"Burns on hit for 12 fire damage over 3 seconds. Two-handed.","stats":[{"id":"dmg","label":"Damage","value":"42–58","delta":12,"deltaText":"+12","better":false},{"id":"speed","label":"Attack speed","value":"1.1","delta":-0.2,"deltaText":"−0.2","better":false},{"id":"cd","label":"Cooldown","value":"4 s","delta":0,"deltaText":"","better":true}],"compareTitle":"Equipped","compareStats":[{"id":"dmg","label":"Damage","value":"30–46"},{"id":"speed","label":"Attack speed","value":"1.3"},{"id":"cd","label":"Cooldown","value":"4 s"}],"footer":"Sell for 120","hints":[{"id":"equip","text":"Equip","action":"confirm"}]}'>

<div id="TooltipCard" role="tooltip" aria-label={title} aria-description={body}>
  {#if compareTitle != ""}
    <section id="Compare" part="TooltipCard.frame" class="card compare">
      <span id="Eyebrow">{compareTitle}</span>
      <div class="rule"></div>
      <ul class="stats">
        {#each compareStats as s (s.id)}
          <li class="stat" part="TooltipCard.row">
            <span class="label" part="TooltipCard.label">{s.label}</span>
            <span class="value" part="TooltipCard.figure">{s.value}</span>
          </li>
        {/each}
      </ul>
    </section>
  {/if}
  <section id="Card" part="TooltipCard.frame" class="card" class:common={rarity == "common"} class:uncommon={rarity == "uncommon"} class:rare={rarity == "rare"} class:epic={rarity == "epic"} class:legendary={rarity == "legendary"} class:mythic={rarity == "mythic"}>
    <header id="Header">
      <div id="Well" part="TooltipCard.well" class="well" hidden={!image && icon == ""}>
        <img id="Art" src={image} alt="" hidden={!image}>
        <og-icon id="Icon" class="glyph" icon={icon} hidden={!!image}></og-icon>
      </div>
      <div id="Names">
        <h3 id="Title" part="TooltipCard.title">{title}</h3>
        <div id="Sub" hidden={subtitle == ""}>
          <div id="Pips">
            <span class="pip" hidden={rarity == "none" || rarity == ""}></span>
            <span class="pip" hidden={rarity == "none" || rarity == "" || rarity == "common"}></span>
            <span class="pip" hidden={rarity != "rare" && rarity != "epic" && rarity != "legendary" && rarity != "mythic"}></span>
            <span class="pip" hidden={rarity != "epic" && rarity != "legendary" && rarity != "mythic"}></span>
            <span class="pip" hidden={rarity != "legendary" && rarity != "mythic"}></span>
            <span class="pip" hidden={rarity != "mythic"}></span>
          </div>
          <span id="Subtitle">{subtitle}</span>
        </div>
      </div>
    </header>
    <div class="rule"></div>
    <ul class="stats" hidden={stats.length == 0}>
      {#each stats as s (s.id)}
        <li class="stat" part="TooltipCard.row">
          <span class="label" part="TooltipCard.label">{s.label}</span>
          <span class="value" part="TooltipCard.figure">{s.value}</span>
          <div class="delta" class:good={s.better ? s.delta < 0 : s.delta > 0} class:bad={s.better ? s.delta > 0 : s.delta < 0} hidden={s.delta == 0}>
            <svg class="mark" class:down={s.delta < 0} viewBox="0 0 8 7"><polygon points="4,0 8,7 0,7"></polygon></svg>
            <span class="amount">{s.deltaText}</span>
          </div>
        </li>
      {/each}
    </ul>
    <p id="Body" hidden={body == ""}>{body}</p>
    <p id="Footer" hidden={footer == ""}>{footer}</p>
    <ul id="Hints" hidden={hints.length == 0}>
      {#each hints as h (h.id)}
        <li class="hint" part="TooltipCard.badge"><kbd class="key" data-action={h.action} hidden={h.action == ""}></kbd><span>{h.text}</span></li>
      {/each}
    </ul>
  </section>
</div>

<style>
#TooltipCard { display: flex; flex-direction: row; align-items: start; gap: var(--space-sm); --rarity: var(--line-strong); }
[part="TooltipCard.frame"] { display: flex; flex-direction: column; width: var(--measure-md); gap: var(--space-sm); }
.common { --rarity: var(--common, var(--line-strong)); }
.uncommon { --rarity: var(--uncommon, var(--success)); }
.rare { --rarity: var(--rare, var(--info)); }
.epic { --rarity: var(--epic, var(--xp)); }
.legendary { --rarity: var(--legendary, var(--warn)); }
.mythic { --rarity: var(--mythic, var(--danger)); }
/* A rarity is the frame's rim and the title's ink, through their tokens, so a pack's frame still dresses it. */
#Card.common, #Card.uncommon, #Card.rare, #Card.epic, #Card.legendary, #Card.mythic { --frame-rim: var(--rarity); --frame-rim-width: var(--stroke-thin); --title-ink: var(--rarity); }
#Eyebrow { font: var(--text-sm); text-transform: uppercase; color: var(--ink-muted); }
#Header { display: flex; align-items: center; gap: var(--space-sm); width: stretch; }
[part="TooltipCard.well"] { display: flex; width: var(--size-touch); height: var(--size-touch); align-items: center; justify-content: center; overflow: hidden; }
#Art { width: var(--size-touch); height: var(--size-touch); }
.glyph { width: var(--size-sm); height: var(--size-sm); color: var(--rarity); }
#Names { display: flex; flex-direction: column; gap: var(--space-xxs); width: stretch; }
[part="TooltipCard.title"] { line-clamp: 2; }
#Sub { display: flex; align-items: center; gap: var(--space-xs); }
#Pips { display: flex; gap: var(--space-xs); }
.pip { width: var(--space-xs); height: var(--space-xs); rotate: 45deg; background: var(--rarity); }
#Subtitle { font: var(--text-sm); color: var(--ink-muted); line-clamp: 2; }
.rule { width: stretch; height: var(--stroke-hairline); background: var(--rarity); opacity: var(--opacity-muted); }
.stats { display: flex; flex-direction: column; gap: var(--space-xxs); width: stretch; margin: 0; padding: 0; list-style: none; }
[part="TooltipCard.row"] { display: flex; align-items: center; gap: var(--space-sm); width: stretch; }
[part="TooltipCard.label"] { width: stretch; font: var(--text-base); font-family: var(--label-font); white-space: nowrap; text-overflow: ellipsis; }
[part="TooltipCard.figure"] { font: var(--text-base); font-family: var(--figure-font); white-space: nowrap; }
.delta { display: flex; align-items: center; gap: var(--space-xxs); }
.good { color: var(--trend-up); }
.bad { color: var(--trend-down); }
.mark { width: var(--space-sm); height: var(--space-sm); fill: currentColor; }
.down { rotate: 180deg; }
.amount { font: var(--text-sm); font-family: var(--figure-font); white-space: nowrap; }
#Body { margin: 0; font: var(--text-base); color: var(--ink); }
#Footer { margin: 0; font: var(--text-sm); color: var(--ink-muted); }
#Hints { display: flex; flex-wrap: wrap; gap: var(--space-sm); margin: 0; padding: 0; list-style: none; }
[part="TooltipCard.badge"] { display: flex; align-items: center; gap: var(--space-xs); padding: var(--space-xxs) var(--space-xs); font: var(--text-sm); white-space: nowrap; }
</style>
