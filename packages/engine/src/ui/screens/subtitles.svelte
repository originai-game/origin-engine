<meta name="screen" content="subtitles" data-layer="hud">
<meta name="parts" content="subtitles.frame subtitles.line subtitles.speaker">
<meta name="prop:lines" content="list" data-doc="The game's own lines, oldest first, a keyed collection of { id, speaker, text }; the game removes a line when it is done. The captions a playing timeline is saying ($captions) follow them, with nothing to pass.">
<meta name="fixture:default" content="{&quot;lines&quot;:[{&quot;id&quot;:&quot;1&quot;,&quot;speaker&quot;:&quot;Keeper&quot;,&quot;text&quot;:&quot;The lamps went dark three nights ago.&quot;}]}">

<div id="Subtitles" part="subtitles.frame" class="bare" role="log" aria-live="polite">
  {#each lines as l (l.id)}
    <div class="line" part="subtitles.line"><p class="speaker" part="subtitles.speaker">{l.speaker}</p><p class="said">{l.text}</p></div>
  {/each}
  {#each $captions as c (c.id)}
    <div class="line" part="subtitles.line"><p class="said">{c.text}</p></div>
  {/each}
</div>

<style>
/* The latest lines at the foot of the screen, centred. */
:root { display: flex; flex-direction: column; justify-content: end; align-items: center; gap: var(--space-xs); block-size: stretch; padding: var(--space-3xl) var(--space-xl); }
.line { display: flex; gap: var(--space-sm); padding: var(--space-xs) var(--space-md); max-width: var(--measure-xxl); background: var(--hud); border-radius: var(--radius-control); }
.speaker, .said { margin: 0; }
.speaker { color: var(--accent); }
</style>
