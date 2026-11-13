<meta name="screen" content="title" data-layer="menu" data-focus="" data-under="none" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./title.en.ogt">
<meta name="parts" content="title.frame title.logo title.prompt">
<meta name="prop:version" content="string" data-doc="The build's version.">
<meta name="prop:legal" content="string" data-doc="The legal line at the foot (copyright, trademarks).">
<meta name="action:confirm" content="Start">
<meta name="fixture:default" content="{&quot;legal&quot;:&quot;© Studio. All rights reserved.&quot;,&quot;version&quot;:&quot;1.0.0&quot;}">

<stack id="Title" part="title.frame" class="bare" interactive="true" onclick={start} onconfirm={start}>
  <art id="Art" kind="keyArt"></art>
  <column id="Background">{@render background?.()}</column>
  <column id="Foot">
    <spacer id="Node4"></spacer>
    <art id="Logo" part="title.logo" kind="logo"></art>
    <spacer id="Node6"></spacer>
    <text id="Prompt" part="title.prompt">{t("screen.title.press")}</text>
    <text id="Legal">{$legal}</text>
  </column>
  <text id="Version">{$version}</text>
</stack>

<style>
#Title { width: stretch; height: stretch; }
#Art { width: stretch; height: stretch; position: absolute; inset-inline-start: 0; top: 0; inset-inline-end: 0; bottom: 0; }
#Background { width: stretch; height: stretch; position: absolute; inset-inline-start: 0; top: 0; inset-inline-end: 0; bottom: 0; }
#Foot { width: stretch; height: stretch; padding: var(--space-3xl); gap: var(--space-lg); align-items: center; }
@media (width < 600px), (height < 600px) {
#Foot { padding: var(--space-xl); }
}
#Logo { width: var(--measure-lg); height: var(--size-4xl); }
@media (width < 600px), (height < 600px) {
#Logo { width: var(--size-7xl); height: var(--size-xxl); }
}
#Prompt { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); text-align: center; animation: breathe calc(var(--motion-named-playful-duration) * 3) var(--motion-easing-in-out) forwards; }
@keyframes breathe {
  0%, 100% { opacity: 1; }
  50% { opacity: var(--motion-scale-breathe); }
}
#Legal { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); text-align: center; }
#Version { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); position: absolute; inset-inline-end: var(--space-lg); bottom: var(--space-md); }
</style>
