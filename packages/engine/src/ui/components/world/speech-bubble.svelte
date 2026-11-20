<meta name="component" content="SpeechBubble">
<meta name="strings" content="./world.en.ogt">
<meta name="prop:entity" content="number" data-doc="Who speaks, by its bits (0: use at).">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="A world point it speaks from without an entity (x y z).">
<meta name="prop:height" content="number" data-doc="World units above the point (over the head).">
<meta name="prop:speaker" content="string" data-doc="Who speaks, over the line in their reaction's ink; empty is none.">
<meta name="prop:reaction" content="string" data-doc="The speaker's colour: friendly (empty), hostile, neutral or player.">
<meta name="prop:text" content="string" data-doc="The line; it wraps at 27 characters.">
<meta name="prop:shout" content="bool" data-doc="A shout: a warm rim and a bolder line.">
<meta name="prop:left" content="number" data-default="-1" data-doc="How much of the line's time is left, 0 to 1, a thin line along the bubble's foot; negative draws none. The game times the line out and removes the bubble.">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","height":2.2,"speaker":"Mira","reaction":"","text":"The Colossus guards the gate. Break its shield, then strike the runes!","shout":false,"left":0.64}'>
<meta name="fixture:short" content='{"text":"Over here!","left":0.3}'>
<meta name="fixture:shout" content='{"speaker":"Vorric","reaction":"hostile","text":"Kneel before the ash!","shout":true,"left":0.8}'>
<meta name="fixture:nameless" content='{"speaker":"","text":"Wait for me at the bridge.","left":-1}'>
<meta name="fixture:overflow" content='{"speaker":"Keeper of the Northern Lighthouse","text":"The lamps went dark three nights ago, and since then the ships have stopped coming; cross the gate before the tide turns.","left":0.12}'>

<!--
  A line spoken over a head: the HUD's glass with a cut corner, the
  speaker in their colour, the line wrapping at 27 characters, a tail to
  the speaker and a thin line of the time left. It pops from its tail.
  Read only; the game decides when it shows and when it goes.
-->
<anchor id="SpeechBubble" entity={entity || null} at={at} offset={"0 " + height + " 0"} declutter="shift" occlusion="none" priority="200">
  <div id="Bubble" part="frame" variant="bare" role="status" aria-label={speaker != '' ? t("world.speech.name", { speaker: speaker }) : text} class:shout={shout} class:reaction-hostile={(reaction == '' ? 'friendly' : reaction) == 'hostile'} class:reaction-neutral={(reaction == '' ? 'friendly' : reaction) == 'neutral'} class:reaction-player={(reaction == '' ? 'friendly' : reaction) == 'player'}>
    <div part="box" class="box">
      {#if speaker != ''}<span part="speaker" class="speaker">{speaker}</span>{/if}
      <p part="line" class="line">{text}</p>
      {#if left >= 0}<div part="timer" class="timer"><div class="left" style:--left={left >= 1 ? 1 : left}></div></div>{/if}
    </div>
    <div part="tail" class="tail"></div>
  </div>
</anchor>

<style>
#Bubble { display: flex; flex-direction: column; align-items: center; color: var(--hud-ink); filter: var(--world-speech-shadow); transform-origin: 50% 100%; animation: var(--world-speech-motion); }
/* The glass with a cut corner; a shout's rim warm. A text box: it grows
   with the text scale, its line wrapping at 27 characters. */
.box { position: relative; display: flex; flex-direction: column; row-gap: calc(2 * var(--font-step-px)); min-inline-size: 0; max-inline-size: min(var(--world-speech-measure), var(--world-speech-room)); padding: calc(7 * var(--font-step-px)) calc(13 * var(--font-step-px)) calc(9 * var(--font-step-px)); box-sizing: border-box; font-size: var(--world-size-speech); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: 0 0 var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.shout .box { box-shadow: var(--world-shout-rim); }
.speaker { min-inline-size: 0; font-size: var(--world-size-title); font-weight: var(--hud-figure-weight); font-stretch: var(--world-caps-stretch); letter-spacing: var(--world-title-tracking); text-transform: var(--hud-label-transform); color: var(--world-friendly-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.reaction-hostile .speaker { color: var(--world-hostile-ink); }
.reaction-neutral .speaker { color: var(--world-neutral-ink); }
.reaction-player .speaker { color: var(--world-player-ink); }
.line { margin: 0; line-height: 1.32; }
.shout .line { font-weight: var(--hud-figure-weight); font-stretch: var(--world-name-stretch); }
/* The time left: a thin line along the foot, draining. */
.timer { position: absolute; left: calc(10 * var(--font-step-px)); right: calc(10 * var(--font-step-px)); bottom: calc(3 * var(--font-step-px)); block-size: var(--stroke-regular); background: var(--world-timer-well); }
.left { block-size: 100%; inline-size: calc(var(--left) * 100%); background: var(--world-timer-fill); }
/* The tail points down to the speaker. */
.tail { flex: none; inline-size: var(--world-tail-width); block-size: var(--world-tail-height); margin-top: calc(-1 * var(--stroke-hairline)); background: var(--hud-tag-fill); clip-path: polygon(0 0, 100% 0, 50% 100%); }
@media (width < 600px) { .box { max-inline-size: min(var(--world-speech-measure), var(--world-speech-room-phone)); } }
@media (height < 500px) { .box { max-inline-size: min(var(--world-speech-measure), var(--world-speech-room-short)); } }
@media (prefers-reduced-motion: reduce) { #Bubble { animation: none; } }
</style>
