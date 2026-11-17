<meta name="component" content="SpeechBubble">
<meta name="parts" content="SpeechBubble.frame SpeechBubble.label SpeechBubble.line SpeechBubble.tail">
<meta name="prop:entity" content="number" data-doc="Who speaks, by its bits (0: use at).">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="A world point it speaks from without an entity (x y z).">
<meta name="prop:height" content="number" data-doc="World units above the point (over the head).">
<meta name="prop:speaker" content="string" data-doc="Who speaks, over the line; empty is none.">
<meta name="prop:text" content="string" data-doc="The line.">
<meta name="prop:shout" content="bool" data-doc="A shout: a jagged warn edge.">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","height":2.2,"speaker":"Mira","text":"The bridge is out. Take the north pass.","shout":false}'>

<anchor id="SpeechBubble" entity={entity || null} at={at} offset={"0 " + height + " 0"} declutter="shift" occlusion="none" priority="200">
  <div id="Bubble" data-layout="stack" data-gap="none" data-motion="pop" class:shout={shout} role="status">
    <div id="Box" part="SpeechBubble.frame" data-layout="stack" data-gap="tight">
      {#if speaker}<span id="Speaker" part="SpeechBubble.label">{speaker}</span>{/if}
      <p id="Line" part="SpeechBubble.line">{text}</p>
    </div>
    <svg id="Tail" part="SpeechBubble.tail" viewBox="0 0 16 10"><path d="M0 0h16L8 10z"></path></svg>
  </div>
</anchor>

<style>
:host { text-shadow: var(--tag-halo); }
#Bubble { align-items: center; }
[part="SpeechBubble.frame"] { max-width: var(--measure-sm); padding: var(--space-sm) var(--space-md); }
/* A shout is the frame's rim, through its tokens, so a pack's frame still dresses it. */
.shout [part="SpeechBubble.frame"] { --frame-rim: var(--warn); --frame-rim-width: var(--stroke-regular); }
.shout [part="SpeechBubble.tail"] { --tail-rim: var(--warn); --tail-rim-width: var(--stroke-regular); }
[part="SpeechBubble.line"] { margin: 0; }
[part="SpeechBubble.tail"] { width: var(--size-xs); height: calc(var(--size-xs) * 0.625); margin-top: calc(var(--stroke-hairline) * -1); }
</style>
