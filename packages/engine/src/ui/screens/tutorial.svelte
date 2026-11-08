<meta name="screen" content="tutorial" data-layer="popover" data-focus="" data-under="none" data-input="menu" data-pause="false" data-capture="false" data-remember="true" data-back="skip">
<meta name="strings" content="./tutorial.en.ogt">
<meta name="prop:title" content="string" data-doc="The step's title.">
<meta name="prop:text" content="string" data-doc="What to do.">
<meta name="prop:step" content="number" data-doc="The step, from 1.">
<meta name="prop:steps" content="number" data-doc="Steps in the lesson.">
<meta name="prop:waits" content="bool" data-doc="The step waits for the player to act (Next hidden).">
<meta name="prop:x" content="number" data-doc="The card's start edge, CSS px from the screen's start (the right, right to left).">
<meta name="prop:y" content="number" data-doc="The card's top, CSS px.">
<meta name="prop:texture" content="number" data-doc="The step's art (0: none).">
<meta name="action:confirm" content="Next">
<meta name="action:back" content="Skip">
<meta name="fixture:default" content="{&quot;step&quot;:2,&quot;steps&quot;:5,&quot;text&quot;:&quot;Press the dodge button just before an attack lands to slip past it.&quot;,&quot;texture&quot;:0,&quot;title&quot;:&quot;Dodge&quot;,&quot;waits&quot;:false,&quot;x&quot;:64,&quot;y&quot;:120}">

<stack id="Tutorial">
  <column id="Highlight">{@render highlight?.()}</column>
  <column id="Card">
    <art id="Art" kind="tutorial" src={$texture} hidden={!(($texture != 0.0))}></art>
    <text id="Title" heading="true">{$title}</text>
    <text id="Text">{$text}</text>
    <column id="Hint">{@render hint?.()}</column>
    <row id="Foot">
      <text id="Step">{t("screen.tutorial.Step", { step: $step, steps: $steps })}</text>
      <spacer id="Node9"></spacer>
      <button id="Skip" variant="quiet" size="small" onclick={skip}>{t("screen.tutorial.skip")}</button>
      <button id="Next" variant="primary" size="small" hidden={!(!($waits))} onclick={next} onconfirm={next}>{t("screen.tutorial.next")}</button>
    </row>
  </column>
</stack>

<style>
#Tutorial { width: stretch; height: stretch; }
#Highlight { width: stretch; height: stretch; position: absolute; inset-inline-start: 0; top: 0; inset-inline-end: 0; bottom: 0; }
#Card { position: absolute; inset-inline-start: bind(x); top: bind(y); width: stretch; max-width: 360px; padding: 16px; gap: 8px; -og-frame: tooltip; -og-enter: pop; }
@media (width < 600px) and (orientation: portrait) {
#Card { inset-inline-start: 12px; inset-inline-end: 12px; top: bind(y); }
}
#Art { width: stretch; }
#Title { font: var(--text-lg); }
#Text { font: var(--text-reading); line-clamp: 5; }
#Hint { width: stretch; }
#Foot { width: stretch; gap: 8px; align-items: center; }
#Step { font: var(--text-caption); color: var(--ink-muted); }
@media (width < 600px), (height < 600px) {
#Skip, #Next { -og-control-size: medium; }
}
</style>
