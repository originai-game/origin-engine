<meta name="screen" content="subtitles" data-layer="hud">
<meta name="prop:lines" content="list" data-doc="The lines shown, oldest first, a keyed collection of { id, speaker, text }; the game removes a line when it is done.">
<meta name="fixture:default" content="{&quot;lines&quot;:[{&quot;id&quot;:&quot;1&quot;,&quot;speaker&quot;:&quot;Keeper&quot;,&quot;text&quot;:&quot;The lamps went dark three nights ago.&quot;}]}">

<column id="Subtitles" role="log" aria-live="polite">
  {#each $lines as l (l.id)}
    <div class="line"><p class="speaker">{l.speaker}</p><p class="said">{l.text}</p></div>
  {/each}
</column>

<style>
#Subtitles { width: stretch; height: stretch; justify-content: end; align-items: center; padding: 48px 24px; gap: 4px; }
.line { display: flex; gap: 6px; padding: 4px 12px; max-width: 720px; background: color-mix(in srgb, var(--backdrop) 72%, transparent); border-radius: var(--radius-control); }
.speaker, .said { margin: 0; }
.speaker { color: var(--accent); }
</style>
