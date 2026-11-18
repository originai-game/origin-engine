<meta name="screen" content="title" data-layer="menu" data-focus="" data-under="none" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./title.en.ogt">
<meta name="parts" content="title.frame title.logo title.prompt title.shade title.rule title.name">
<meta name="prop:version" content="string" data-doc="The build's version.">
<meta name="prop:legal" content="string" data-doc="The legal line at the foot (copyright, trademarks).">
<meta name="prop:keyArt" content="number" data-doc="The key art's texture under the title (og.load.texture; 0: the style's dark ground).">
<meta name="prop:logo" content="number" data-doc="The game's logo texture (og.load.texture; 0: the name set as a wordmark).">
<meta name="prop:name" content="string" data-doc="The game's name, set as a wordmark where there is no logo (empty: the logo slot's placeholder).">
<meta name="action:confirm" content="Start">
<meta name="fixture:default" content="{&quot;keyArt&quot;:0,&quot;legal&quot;:&quot;© Studio. All rights reserved.&quot;,&quot;logo&quot;:0,&quot;name&quot;:&quot;Lantern Coast&quot;,&quot;version&quot;:&quot;1.0.0&quot;}">

<div id="Title" part="title.frame" class="bare" interactive="true" onclick={start} onconfirm={start}>
  {#if keyArt != 0}<art id="Art" kind="keyArt" src={keyArt}></art>{/if}
  <div id="Background">{@render background?.()}</div>
  <div id="Shade" part="title.shade"></div>
  <div id="Foot">
    {#if logo == 0 && name != ""}
      <h1 id="Name" part="title.name">{name}</h1>
    {:else}
      <art id="Logo" part="title.logo" kind="logo" src={logo}></art>
    {/if}
    <div id="Call">
      <span class="rule" part="title.rule"></span>
      <div id="Prompt" part="title.prompt"><p class="press">{t("screen.title.press")}</p><p class="tap">{t("screen.title.tap")}</p></div>
      <span class="rule end" part="title.rule"></span>
    </div>
    <p id="Legal">{legal}</p>
    <p id="Version">{version}</p>
  </div>
</div>

<style>
/* The key art fills the screen under a vignette; the logo (or the name as
   a wordmark) sits in the middle, the call to start between two lit rules
   under it, the legal line and the version at the foot. */
:root { position: relative; block-size: stretch; overflow: hidden; background: var(--front-ground); color: var(--front-ink); }
#Art, #Background, [part="title.shade"] { position: absolute; inset: 0; }
#Art { inline-size: 100%; block-size: 100%; object-fit: cover; }
[part="title.shade"] { background: var(--front-vignette); }
p, h1 { margin: 0; }
#Foot { position: relative; display: grid; grid-template-columns: minmax(0, 1fr); grid-template-rows: 1fr auto 1fr auto auto auto; justify-items: center; row-gap: var(--space-md); block-size: 100%; padding: var(--space-3xl) var(--space-3xl) var(--space-xl); container-type: inline-size; }
[part="title.logo"] { grid-row: 2; inline-size: min(100%, calc(860 * var(--font-step-px))); aspect-ratio: 2.4; block-size: auto; object-fit: contain; filter: var(--front-logo-lift); }
/* The wordmark keeps whole words: it takes a share of the column's width,
   so the longest word of a long name fits on its line at any text scale. */
[part="title.name"] { grid-row: 2; max-inline-size: 100%; font-size: min(calc(76 * var(--font-step-px)), 8cqi); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); font-stretch: var(--front-stretch); letter-spacing: var(--font-tracking-sm); text-transform: uppercase; text-align: center; overflow-wrap: normal; hyphens: none; text-shadow: var(--front-shadow); }
/* The call is a band of one measure: a lit rule each side, the words in
   the room between them. */
#Call { grid-row: 4; display: flex; align-items: center; column-gap: var(--space-lg); inline-size: min(100%, calc(560 * var(--font-step-px))); }
[part="title.rule"] { flex: 0 0 calc(120 * var(--font-step-px)); block-size: var(--stroke-hairline); background: var(--front-rule); box-shadow: var(--front-rule-glow); }
[part="title.rule"].end { scale: -1 1; }
[part="title.prompt"] { flex: 1 1 0; min-inline-size: 0; font-size: calc(18 * var(--font-step-px)); font-weight: var(--front-call-weight); letter-spacing: var(--front-call-tracking); text-transform: uppercase; text-align: center; text-shadow: var(--front-call-glow); animation: breathe calc(var(--motion-named-playful-duration) * 3) var(--motion-easing-in-out) infinite; }
.tap { display: none; }
@keyframes breathe {
  0%, 100% { opacity: 1; }
  50% { opacity: var(--motion-scale-breathe); }
}
@media (prefers-reduced-motion: reduce) { [part="title.prompt"] { animation: none; } }
#Legal { grid-row: 5; font-size: calc(12 * var(--font-step-px)); letter-spacing: var(--font-tracking-sm); color: var(--front-ink-faint); text-align: center; }
#Version { grid-row: 6; justify-self: end; font-size: calc(12 * var(--font-step-px)); font-variant-numeric: tabular-nums; color: var(--front-ink-faint); }
/* On a touch screen the call names the tap. */
@media (pointer: coarse) {
  .press { display: none; }
  .tap { display: block; }
}
@media (width < 600px), (height < 600px) {
  #Foot { padding: var(--space-xxl) var(--space-lg) var(--space-lg); row-gap: var(--space-sm); }
  [part="title.logo"] { inline-size: 86%; }
  [part="title.name"] { font-size: min(calc(44 * var(--font-step-px)), 11cqi); }
  #Call { column-gap: var(--space-sm); inline-size: min(100%, calc(220 * var(--font-step-px))); }
  [part="title.rule"] { flex-basis: calc(32 * var(--font-step-px)); }
  [part="title.prompt"] { font-size: calc(14 * var(--font-step-px)); letter-spacing: var(--font-tracking-wide); }
  #Version { justify-self: center; }
}
</style>
