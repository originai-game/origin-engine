<meta name="screen" content="main-menu" data-layer="menu" data-focus="MenuMenu/Entries/Continue" data-under="none" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./main-menu.en.ogt">
<meta name="prop:canContinue" content="bool" data-doc="A save exists: Continue shows and takes the focus.">
<meta name="prop:saveLine" content="string" data-doc="Under Continue: where the last save stands (chapter, time played).">
<meta name="prop:version" content="string" data-doc="The build's version, shown small in the corner.">
<meta name="prop:news" content="string" data-doc="A line for the news card (empty: no card).">
<meta name="prop:keyArt" content="number" data-doc="The key art's texture behind the menu (og.load.texture; 0: the style's dark ground).">
<meta name="prop:logo" content="number" data-doc="The game's logo texture (og.load.texture; 0: the name set as a wordmark).">
<meta name="prop:name" content="string" data-doc="The game's name, set as a wordmark where there is no logo (empty: the logo slot's placeholder).">
<meta name="action:confirm" content="Select">
<meta name="fixture:default" content="{&quot;canContinue&quot;:true,&quot;keyArt&quot;:0,&quot;logo&quot;:0,&quot;name&quot;:&quot;Lantern Coast&quot;,&quot;news&quot;:&quot;Season 2 is live&quot;,&quot;saveLine&quot;:&quot;Chapter 3 · 4h 12m&quot;,&quot;version&quot;:&quot;1.0.0&quot;}">

<div id="RootMenu">
  {#if keyArt != 0}<art id="Art" kind="keyArt" src={keyArt}></art>{/if}
  <div id="Background">{@render background?.()}</div>
  <div class="shade" id="Shade" part="shade"></div>
  <nav class="frame" id="MenuMenu" part="frame" variant="bare">
    {#if logo == 0 && name != ""}
      <h1 class="name" id="Name" part="name">{name}</h1>
    {:else}
      <art class="logo" id="Logo" part="logo" kind="logo" src={logo}></art>
    {/if}
    {#if news != ""}
      <aside class="news" id="News" part="news" interactive="true" focusable="true" onclick={news}>
        <div id="Words">
          <p id="Eyebrow">{t("screen.main-menu.newsEyebrow")}</p>
          <p id="Line">{news}</p>
        </div>
        <og-icon class="chevron" icon="chevronRight"></og-icon>
      </aside>
    {/if}
    <div id="Entries" class:after-news={news != ""}>
      {#if canContinue}
        <button id="Continue" part="entry" class="entry" onclick={continue}><div class="face">
          <div class="words"><span class="label">{t("screen.main-menu.continue")}</span><span id="SaveLine">{saveLine}</span></div>
          <og-icon class="chevron" icon="chevronRight"></og-icon>
        </div></button>
      {/if}
      <button id="NewGame" part="entry" class="entry" onclick={newGame}><div class="face"><span class="label">{t("screen.main-menu.newGame")}</span><og-icon class="chevron" icon="chevronRight"></og-icon></div></button>
      <button id="Modes" part="entry" class="entry" onclick={modes}><div class="face"><span class="label">{t("screen.main-menu.modes")}</span><og-icon class="chevron" icon="chevronRight"></og-icon></div></button>
      <button id="Settings" part="entry" class="entry" onclick={settings}><div class="face"><span class="label">{t("screen.main-menu.settings")}</span><og-icon class="chevron" icon="chevronRight"></og-icon></div></button>
      <hr id="Divider">
      <button id="Credits" part="entry" class="entry minor" onclick={credits}><div class="face"><span class="label">{t("screen.main-menu.credits")}</span></div></button>
      <button id="Quit" part="entry" class="entry minor" onclick={quit}><div class="face"><span class="label">{t("screen.main-menu.quit")}</span></div></button>
      <div id="Extra">{@render extra?.()}</div>
    </div>
  </nav>
  <p class="version" id="Version" part="version">{version}</p>
</div>

<style>
#RootMenu { position: relative; block-size: stretch; overflow: hidden; background: var(--front-ground); color: var(--front-ink); }
#Art, #Background, .shade { position: absolute; inset: 0; }
#Art { inline-size: 100%; block-size: 100%; object-fit: cover; object-position: 62% 50%; }
.shade { background: var(--front-shade); }
p, h1 { margin: 0; }
/* The logo at the top, the entries at the foot of a column down the
   leading side. Taller than the screen (a short landscape phone, a large
   text scale, a game's extra rows) the column starts at the top and
   scrolls, so no logo or entry is cut off out of reach. */
.frame { position: relative; display: flex; flex-direction: column; inline-size: 100%; block-size: 100%; padding: var(--space-3xl) var(--space-xxl) var(--space-xxl) var(--space-3xl); overflow: hidden auto; }
#Name, #Logo, #News, #Entries { flex-shrink: 0; }
.logo { align-self: start; inline-size: min(100%, calc(380 * var(--font-step-px))); aspect-ratio: 2.4; block-size: auto; object-fit: contain; filter: var(--front-logo-lift); }
.name { align-self: start; max-inline-size: 100%; font-size: calc(48 * var(--font-step-px)); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); font-stretch: var(--front-stretch); letter-spacing: var(--font-tracking-sm); text-transform: uppercase; overflow-wrap: anywhere; text-shadow: var(--front-shadow); }
#Entries { display: flex; flex-direction: column; row-gap: var(--space-xxs); inline-size: calc(440 * var(--font-step-px)); max-inline-size: 100%; margin-block-start: auto; margin-inline-start: calc(var(--space-lg) * -1); }
/* An entry is type, not a box: dim until it is the one under the pointer
   or the focus, then the accent washes it from its leading edge, which
   carries a lit bar, and its chevron shows. */
button.entry { --button-font-size: var(--front-entry-size); --button-text-shadow: var(--front-shadow); --button-ink: var(--front-ink-dim); --button-hover-ink: var(--front-ink); --button-quiet-ink: var(--front-ink-dim); --button-quiet-hover-ink: var(--front-ink); inline-size: stretch; min-block-size: var(--touch-target); background: transparent; box-shadow: none; border-radius: 0; padding: var(--space-sm) var(--space-lg); }
button.entry { background: transparent; box-shadow: none; border-radius: 0; padding: var(--space-sm) var(--space-lg); color: var(--front-ink-dim); }
button.entry:hover, button.entry:focus-visible { background: var(--front-entry-on-fill); box-shadow: var(--front-entry-on-edge); color: var(--front-ink); }
button.entry:active { background: var(--front-entry-press-fill); box-shadow: var(--front-entry-on-edge); color: var(--front-ink); }
.face { display: flex; align-items: center; column-gap: var(--space-md); inline-size: stretch; }
.words { display: flex; flex-direction: column; flex: 1 1 auto; min-inline-size: 0; }
.label { flex: 1 1 auto; min-inline-size: 0; text-align: start; font-size: var(--front-entry-size); line-height: var(--font-leading-tight); font-weight: var(--front-entry-weight); text-shadow: var(--front-shadow); overflow-wrap: anywhere; }
#SaveLine { text-align: start; font-size: calc(14 * var(--font-step-px)); font-weight: var(--font-weight-medium); color: var(--front-ink-faint); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.entry .chevron { flex: none; inline-size: var(--size-xs); block-size: var(--size-xs); color: var(--front-accent); opacity: 0; }
button.entry:hover .chevron, button.entry:focus-visible .chevron { opacity: 1; }
button.entry:hover #SaveLine, button.entry:focus-visible #SaveLine { color: var(--front-ink-dim); }
.chevron:dir(rtl) { scale: -1 1; }
#Divider { block-size: var(--stroke-hairline); margin: var(--space-sm) 0 var(--space-xs) var(--space-lg); border: 0; background: var(--front-divider); }
button.entry.minor { padding-block: var(--space-xs); }
button.entry.minor { --button-font-size: var(--front-entry-quiet-size); --button-ink: var(--front-ink-faint); --button-quiet-ink: var(--front-ink-faint); }
button.entry.minor { color: var(--front-ink-faint); }
/* News: a cut glass plate in the far corner (on a phone, above the entries). */
.news { position: absolute; inset-inline-end: var(--space-3xl); inset-block-end: var(--space-3xl); display: flex; align-items: center; column-gap: var(--space-md); inline-size: calc(360 * var(--font-step-px)); max-inline-size: calc(100% - 2 * var(--space-3xl)); padding: var(--space-md) var(--space-lg); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--front-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.news:hover, .news:focus-visible { translate: 0 var(--state-lift); box-shadow: var(--hud-plate-lit-rim); }
.news:active { translate: 0 0; }
#Words { display: flex; flex-direction: column; row-gap: var(--space-xxs); flex: 1 1 auto; min-inline-size: 0; }
#Eyebrow { font-size: calc(12 * var(--font-step-px)); font-weight: var(--font-weight-bold); letter-spacing: var(--front-call-tracking); text-transform: uppercase; color: var(--front-accent); }
#Line { font-size: calc(20 * var(--font-step-px)); line-height: var(--font-leading-snug); font-weight: var(--font-weight-semibold); line-clamp: 2; }
#News .chevron { flex: none; inline-size: var(--size-xs); block-size: var(--size-xs); color: var(--front-ink-dim); }
.version { position: absolute; inset-inline-end: var(--space-lg); inset-block-end: var(--space-md); font-size: calc(12 * var(--font-step-px)); font-variant-numeric: tabular-nums; color: var(--front-ink-faint); }
.shade:size(compact) { background: var(--front-shade-compact); }
#Art:size(compact) { object-position: 80% 50%; }
.frame:size(compact) { inline-size: auto; padding: var(--space-xxl) var(--space-lg) var(--space-xl); }
.logo:size(compact) { align-self: center; inline-size: 78%; }
.name:size(compact) { align-self: center; text-align: center; font-size: calc(36 * var(--font-step-px)); }
.news:size(compact) { position: static; inline-size: auto; max-inline-size: none; margin-block: auto var(--space-md); padding: var(--space-sm) var(--space-md); }
#Entries.after-news:size(compact) { margin-block-start: 0; }
#Line:size(compact) { font-size: calc(16 * var(--font-step-px)); }
#Entries:size(compact) { inline-size: auto; max-inline-size: none; margin-inline: calc(var(--space-lg) * -1); }
button.entry:size(compact) { padding-inline: var(--space-xl) var(--space-lg); }
button.entry:size(compact) { --button-font-size: var(--front-entry-size-compact); }
button.entry.minor:size(compact) { --button-font-size: calc(16 * var(--font-step-px)); }
#Divider:size(compact) { margin-inline-start: var(--space-xl); }
.version:size(compact) { inset-inline-end: var(--space-md); inset-block-end: var(--space-xs); }
.frame:size(short) { padding: var(--space-md) var(--space-lg); }
.logo:size(short), .name:size(short) { display: none; }
.news:size(short) { display: none; }
#Entries:size(short) { row-gap: 0; }
</style>
