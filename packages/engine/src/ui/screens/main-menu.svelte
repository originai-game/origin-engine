<meta name="screen" content="main-menu" data-layer="menu" data-focus="Menu/Continue" data-under="none" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./main-menu.en.ogt">
<meta name="parts" content="main-menu.frame main-menu.logo main-menu.version">
<meta name="prop:canContinue" content="bool" data-doc="A save exists: Continue shows and takes the focus.">
<meta name="prop:saveLine" content="string" data-doc="Under Continue: where the last save stands (chapter, time played).">
<meta name="prop:version" content="string" data-doc="The build's version, shown small in the corner.">
<meta name="prop:news" content="string" data-doc="A line for the news card (empty: no card).">
<meta name="action:confirm" content="Select">
<meta name="fixture:default" content="{&quot;canContinue&quot;:true,&quot;news&quot;:&quot;Season 2 is live&quot;,&quot;saveLine&quot;:&quot;Chapter 3 · 4h 12m&quot;,&quot;version&quot;:&quot;1.0.0&quot;}">

<div id="RootMenu">
  <art id="Art" kind="keyArt"></art>
  <div id="Background">{@render background?.()}</div>
  <nav id="MenuMenu" part="main-menu.frame" class="bare">
    <art id="Logo" part="main-menu.logo" kind="logo"></art>
    {#if canContinue}
      <button id="Continue" variant="primary" class="large" onclick={continue}>{t("screen.main-menu.continue")}</button>
      <p id="SaveLine">{saveLine}</p>
    {/if}
    <button id="NewGame" class="large" onclick={newGame}>{t("screen.main-menu.newGame")}</button>
    <button id="Modes" class="large" onclick={modes}>{t("screen.main-menu.modes")}</button>
    <button id="Settings" class="large" onclick={settings}>{t("screen.main-menu.settings")}</button>
    <button id="Credits" variant="quiet" onclick={credits}>{t("screen.main-menu.credits")}</button>
    <button id="Quit" variant="quiet" onclick={quit}>{t("screen.main-menu.quit")}</button>
    <div id="Extra">{@render extra?.()}</div>
  </nav>
  {#if news != ""}
    <aside id="News" interactive="true" focusable="true" onclick={news}>
      <p id="Eyebrow">{t("screen.main-menu.newsEyebrow")}</p>
      <p id="Line">{news}</p>
    </aside>
  {/if}
  <p id="Version" part="main-menu.version">{version}</p>
</div>

<style>
:root { position: relative; block-size: stretch; }
#Art, #Background { position: absolute; inset: 0; }
p { margin: 0; }
/* The logo at the top, the buttons centred in the room under it. Taller
   than the screen (a short landscape phone, a large text scale, a game's
   extra rows) the column starts at the top and scrolls, so no logo or
   button is cut off out of reach. */
[part="main-menu.frame"] { position: relative; display: flex; flex-direction: column; gap: var(--space-md); inline-size: fit-content; min-inline-size: var(--measure-lg); block-size: 100%; padding: var(--space-3xl); overflow-y: auto; }
[part="main-menu.logo"] { align-self: start; inline-size: var(--size-7xl); block-size: var(--size-3xl); }
[part="main-menu.logo"] + * { margin-block-start: auto; }
#Extra { margin-block-end: auto; }
.large, .large::part(base) { block-size: var(--control-lg); }
#SaveLine, #Version { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#News { position: absolute; inset-inline-end: var(--space-xxl); inset-block-end: var(--space-xxl); display: flex; flex-direction: column; gap: var(--space-xs); padding: var(--space-lg); background: var(--panel-raised); border-radius: var(--radius-card); outline: var(--stroke-hairline) solid var(--line); outline-offset: calc(var(--stroke-hairline) * -1); }
#News:hover, #News:focus-visible { translate: 0 var(--state-lift); }
#News:active { translate: 0 0; }
/* Emphasised ink (--highlight), not --primary: that is a fill for the
   directed action, which a pack may make a deep colour no text reads in. */
#Eyebrow { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--highlight); }
#Line { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); line-clamp: 2; }
#Version { position: absolute; inset-inline-start: var(--space-lg); inset-block-end: var(--space-md); }
@media (width < 600px), (height < 600px) {
  [part="main-menu.frame"] { inline-size: auto; min-inline-size: 0; padding: var(--space-xl); }
  [part="main-menu.logo"] { inline-size: var(--size-6xl); block-size: var(--size-xxl); }
  #News { display: none; }
}
@media (height < 600px) and (orientation: landscape) {
  [part="main-menu.frame"] { padding: var(--space-md); gap: var(--space-xs); }
  [part="main-menu.logo"] { display: none; }
  .large, .large::part(base) { block-size: var(--control-md); }
}
</style>
