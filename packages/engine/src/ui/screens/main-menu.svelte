<meta name="screen" content="main-menu" data-layer="menu" data-focus="Menu/Continue" data-under="none" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./main-menu.en.ogt">
<meta name="parts" content="main-menu.frame main-menu.logo main-menu.version">
<meta name="prop:canContinue" content="bool" data-doc="A save exists: Continue shows and takes the focus.">
<meta name="prop:saveLine" content="string" data-doc="Under Continue: where the last save stands (chapter, time played).">
<meta name="prop:version" content="string" data-doc="The build's version, shown small in the corner.">
<meta name="prop:news" content="string" data-doc="A line for the news card (empty: no card).">
<meta name="action:confirm" content="Select">
<meta name="fixture:default" content="{&quot;canContinue&quot;:true,&quot;news&quot;:&quot;Season 2 is live&quot;,&quot;saveLine&quot;:&quot;Chapter 3 · 4h 12m&quot;,&quot;version&quot;:&quot;1.0.0&quot;}">

<stack id="RootMenu">
  <art id="Art" kind="keyArt"></art>
  <column id="Background">{@render background?.()}</column>
  <column id="MenuMenu" part="main-menu.frame" class="bare">
    <art id="Logo" part="main-menu.logo" kind="logo"></art>
    <spacer id="Node5"></spacer>
    <button id="Continue" variant="primary" class="large" hidden={!($canContinue)} onclick={continue}>{t("screen.main-menu.continue")}</button>
    <text id="SaveLine" hidden={!($canContinue)}>{$saveLine}</text>
    <button id="NewGame" class="large" onclick={newGame}>{t("screen.main-menu.newGame")}</button>
    <button id="Modes" class="large" onclick={modes}>{t("screen.main-menu.modes")}</button>
    <button id="Settings" class="large" onclick={settings}>{t("screen.main-menu.settings")}</button>
    <button id="Credits" variant="quiet" onclick={credits}>{t("screen.main-menu.credits")}</button>
    <button id="Quit" variant="quiet" onclick={quit}>{t("screen.main-menu.quit")}</button>
    <column id="Extra">{@render extra?.()}</column>
    <spacer id="Node14"></spacer>
  </column>
  <panel id="News" interactive="true" focusable="true" hidden={!(($news != ""))} onclick={news}>
    <text id="Eyebrow">{t("screen.main-menu.newsEyebrow")}</text>
    <text id="Line">{$news}</text>
  </panel>
  <text id="Version" part="main-menu.version">{$version}</text>
</stack>

<style>
.large, .large::part(base) { height: var(--control-lg); }
@media (height < 600px) and (orientation: landscape) {
.large, .large::part(base) { height: var(--control-md); }
}
#RootMenu { width: stretch; height: stretch; }
#Art { width: stretch; height: stretch; position: absolute; inset-inline-start: 0; top: 0; inset-inline-end: 0; bottom: 0; }
#Background { width: stretch; height: stretch; position: absolute; inset-inline-start: 0; top: 0; inset-inline-end: 0; bottom: 0; }
#MenuMenu { width: fit-content; min-width: var(--measure-lg); height: stretch; padding: var(--space-3xl); gap: var(--space-md); justify-content: center; }
@media (width < 600px), (height < 600px) {
#MenuMenu { padding: var(--space-xl); width: stretch; min-width: 0; }
}
@media (height < 600px) and (orientation: landscape) {
#MenuMenu { padding: var(--space-md); gap: var(--space-xs); }
}
#Logo { width: var(--size-7xl); height: var(--size-3xl); }
@media (width < 600px), (height < 600px) {
#Logo { width: var(--size-6xl); height: var(--size-xxl); }
}
@media (height < 600px) and (orientation: landscape) {
#Logo { display: none; }
}
#Continue { width: stretch; }
#SaveLine { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#NewGame { width: stretch; }
#Modes { width: stretch; }
#Settings { width: stretch; }
#Credits { width: stretch; }
#Quit { width: stretch; }
#Extra { width: stretch; }
#News { position: absolute; inset-inline-end: var(--space-xxl); bottom: var(--space-xxl); padding: var(--space-lg); gap: var(--space-xs); background: var(--panel-raised); border-radius: var(--radius-card); outline: var(--stroke-hairline) solid var(--line); outline-offset: calc(var(--stroke-hairline) * -1); }
@media (width < 600px), (height < 600px) {
#News { display: none; }
}
#News { translate: 0 0; }
#News:hover { translate: 0 var(--state-lift); }
#News:active { translate: 0 0; }
#News:focus-visible { translate: 0 var(--state-lift); }
#Eyebrow { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--primary); }
#Line { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); line-clamp: 2; }
#Version { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); position: absolute; inset-inline-start: var(--space-lg); bottom: var(--space-md); }
</style>
