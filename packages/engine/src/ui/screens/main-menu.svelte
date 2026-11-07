<meta name="screen" content="main-menu" data-layer="menu" data-focus="Menu/Continue" data-under="none" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./main-menu.en.ogt">
<meta name="prop:canContinue" content="bool" data-doc="A save exists: Continue shows and takes the focus.">
<meta name="prop:saveLine" content="string" data-doc="Under Continue: where the last save stands (chapter, time played).">
<meta name="prop:version" content="string" data-doc="The build's version, shown small in the corner.">
<meta name="prop:news" content="string" data-doc="A line for the news card (empty: no card).">
<meta name="action:confirm" content="Select">
<meta name="fixture:default" content="{&quot;canContinue&quot;:true,&quot;news&quot;:&quot;Season 2 is live&quot;,&quot;saveLine&quot;:&quot;Chapter 3 · 4h 12m&quot;,&quot;version&quot;:&quot;1.0.0&quot;}">

<stack id="RootMenu">
  <art id="Art" kind="keyArt"></art>
  <column id="Background">{@render background?.()}</column>
  <column id="MenuMenu">
    <art id="Logo" kind="logo"></art>
    <spacer id="Node5"></spacer>
    <button id="Continue" variant="primary" size="large" hidden={!($canContinue)} onclick={continue}>{t("screen.main-menu.continue")}</button>
    <text id="SaveLine" hidden={!($canContinue)}>{$saveLine}</text>
    <button id="NewGame" size="large" onclick={newGame}>{t("screen.main-menu.newGame")}</button>
    <button id="Modes" size="large" onclick={modes}>{t("screen.main-menu.modes")}</button>
    <button id="Settings" size="large" onclick={settings}>{t("screen.main-menu.settings")}</button>
    <button id="Credits" variant="quiet" onclick={credits}>{t("screen.main-menu.credits")}</button>
    <button id="Quit" variant="quiet" onclick={quit}>{t("screen.main-menu.quit")}</button>
    <column id="Extra">{@render extra?.()}</column>
    <spacer id="Node14"></spacer>
  </column>
  <panel id="News" interactive="true" focusable="true" hidden={!(($news != ""))} onclick={news}>
    <text id="Eyebrow">{t("screen.main-menu.newsEyebrow")}</text>
    <text id="Line">{$news}</text>
  </panel>
  <text id="Version">{$version}</text>
</stack>

<style>
#RootMenu { width: stretch; height: stretch; }
#Art { width: stretch; height: stretch; position: absolute; inset-inline-start: 0; top: 0; inset-inline-end: 0; bottom: 0; }
#Background { width: stretch; height: stretch; position: absolute; inset-inline-start: 0; top: 0; inset-inline-end: 0; bottom: 0; }
#MenuMenu { width: fit-content; min-width: 420px; height: stretch; padding: 48px; gap: 12px; justify-content: center; }
@media phonePortrait phoneLandscape {
#MenuMenu { padding: 24px; width: stretch; min-width: 0; }
}
@media phoneLandscape {
#MenuMenu { padding: 12px; gap: 4px; }
}
#Logo { width: 320px; height: 120px; }
@media phonePortrait phoneLandscape {
#Logo { width: 240px; height: 90px; }
}
@media phoneLandscape {
#Logo { display: none; }
}
#Continue { width: stretch; }
@media phoneLandscape {
#Continue { -og-control-size: medium; }
}
#SaveLine { font: var(--text-caption); color: var(--ink-muted); }
#NewGame { width: stretch; }
@media phoneLandscape {
#NewGame { -og-control-size: medium; }
}
#Modes { width: stretch; }
@media phoneLandscape {
#Modes { -og-control-size: medium; }
}
#Settings { width: stretch; }
@media phoneLandscape {
#Settings { -og-control-size: medium; }
}
#Credits { width: stretch; }
#Quit { width: stretch; }
#Extra { width: stretch; }
#News { position: absolute; inset-inline-end: 32px; bottom: 32px; padding: 16px; gap: 4px; background: var(--panel-raised); -og-frame: card; }
@media phonePortrait phoneLandscape {
#News { display: none; }
}
#News { translate: 0 0; }
#News:hover { translate: 0 -2px; }
#News:active { translate: 0 0; }
#News:focus-visible { translate: 0 -2px; }
#Eyebrow { font: var(--text-caption); color: var(--primary); }
#Line { font: var(--text-lg); line-clamp: 2; }
#Version { font: var(--text-caption); color: var(--ink-muted); position: absolute; inset-inline-start: 16px; bottom: 12px; }
</style>
