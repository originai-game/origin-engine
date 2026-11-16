<meta name="screen" content="title" data-layer="menu" data-focus="" data-under="none" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./title.en.ogt">
<meta name="parts" content="title.frame title.logo title.prompt">
<meta name="prop:version" content="string" data-doc="The build's version.">
<meta name="prop:legal" content="string" data-doc="The legal line at the foot (copyright, trademarks).">
<meta name="action:confirm" content="Start">
<meta name="fixture:default" content="{&quot;legal&quot;:&quot;© Studio. All rights reserved.&quot;,&quot;version&quot;:&quot;1.0.0&quot;}">

<div id="Title" part="title.frame" class="bare" interactive="true" onclick={start} onconfirm={start}>
  <art id="Art" kind="keyArt"></art>
  <div id="Background">{@render background?.()}</div>
  <div id="Foot">
    <art id="Logo" part="title.logo" kind="logo"></art>
    <p id="Prompt" part="title.prompt">{t("screen.title.press")}</p>
    <p id="Legal">{legal}</p>
  </div>
  <p id="Version">{version}</p>
</div>

<style>
/* Key art and the game's background fill the screen; the logo sits in the
   middle, the prompt and legal line under it. */
:root { position: relative; block-size: stretch; }
#Art, #Background { position: absolute; inset: 0; }
p { margin: 0; }
#Foot { position: relative; display: grid; grid-template-rows: 1fr auto 1fr auto auto; justify-items: center; gap: var(--space-lg); block-size: 100%; padding: var(--space-3xl); }
[part="title.logo"] { grid-row: 2; inline-size: var(--measure-lg); block-size: var(--size-4xl); }
[part="title.prompt"] { grid-row: 4; font: var(--text-lg); letter-spacing: var(--type-lg-tracking); text-align: center; animation: breathe calc(var(--motion-named-playful-duration) * 3) var(--motion-easing-in-out) forwards; }
@keyframes breathe {
  0%, 100% { opacity: 1; }
  50% { opacity: var(--motion-scale-breathe); }
}
#Legal { grid-row: 5; font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); text-align: center; }
#Version { position: absolute; inset-inline-end: var(--space-lg); inset-block-end: var(--space-md); font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
@media (width < 600px), (height < 600px) {
  #Foot { padding: var(--space-xl); }
  [part="title.logo"] { inline-size: var(--size-7xl); block-size: var(--size-xxl); }
}
</style>
