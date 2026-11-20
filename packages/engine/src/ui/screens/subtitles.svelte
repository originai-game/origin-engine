<meta name="screen" content="subtitles" data-layer="hud">
<meta name="prop:lines" content="list" data-doc="The game's own lines, oldest first, a keyed collection of { id, speaker, text }; the game removes a line when it is done. The captions a playing timeline is saying ($captions) follow them, with nothing to pass.">
<meta name="fixture:default" content="{&quot;lines&quot;:[{&quot;id&quot;:&quot;1&quot;,&quot;speaker&quot;:&quot;Keeper&quot;,&quot;text&quot;:&quot;The lamps went dark three nights ago.&quot;}]}">

<!--
  The latest lines at the foot of the screen, centred, in HUD type: the
  speaker in gold small capitals over the words in white, each with one
  stroke under the fill and a soft shadow, on a soft dark band that holds
  them over the brightest sky. Lines wrap balanced, so no word is left
  alone; an older line steps back, smaller (still white, so it holds on a bright sky).
-->
<div id="Subtitles" part="frame" variant="bare" role="log" aria-live="polite">
  <div id="Lines">
    {#each lines as l, i (l.id)}
      <div class="line" part="line" class:past={i < lines.length - 1}><p class="speaker" part="speaker">{l.speaker}</p><p class="said">{l.text}</p></div>
    {/each}
    {#each $captions as c, i (c.id)}
      <div class="line" part="line" class:past={i < $captions.length - 1}><p class="said">{c.text}</p></div>
    {/each}
  </div>
</div>

<style>
/* HUD type over the scene: one stroke under the fill and one soft shadow. */
#Subtitles { color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--alert-stroke-color); paint-order: stroke fill; }
#Subtitles { display: flex; flex-direction: column; justify-content: end; align-items: center; block-size: stretch; padding: var(--space-3xl) calc(env(-og-hud-inset-right) + var(--space-xl)) calc(env(-og-hud-inset-bottom) + var(--space-3xl)) calc(env(-og-hud-inset-left) + var(--space-xl)); }
#Lines { display: flex; flex-direction: column; align-items: center; gap: var(--space-sm); max-inline-size: var(--measure-xxl); padding: var(--space-xl) var(--space-3xl); background: var(--alert-subtitle-shade); }
.line { display: flex; flex-direction: column; align-items: center; gap: var(--space-xxs); text-align: center; }
.speaker, .said { margin: 0; }
.speaker { font-size: var(--hud-size-name); font-weight: var(--alert-weight); font-variant-caps: var(--alert-speaker-caps); letter-spacing: var(--hud-label-tracking); color: var(--hud-gold); }
.said { font-size: var(--alert-size-said); font-weight: var(--hud-label-weight); line-height: var(--font-leading-snug); text-wrap: balance; overflow-wrap: anywhere; }
/* An older line steps back. */
.line.past .said { font-size: var(--hud-size-name); }
#Lines:size(compact) { padding-inline: var(--space-md); }
.said:size(compact) { font-size: var(--alert-size-said-compact); }
</style>
