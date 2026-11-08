<meta name="component" content="SpeechBubble">
<meta name="prop:entity" content="number" data-doc="Who speaks, by its bits (0: use at).">
<meta name="prop:at" content="string" data-doc="A world point it speaks from without an entity (x y z).">
<meta name="prop:height" content="number" data-doc="World units above the point (over the head).">
<meta name="prop:speaker" content="string" data-doc="Who speaks, over the line; empty is none.">
<meta name="prop:text" content="string" data-doc="The line.">
<meta name="prop:shout" content="bool" data-doc="A shout: a jagged warn edge.">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","height":2.2,"speaker":"Mira","text":"The bridge is out. Take the north pass.","shout":false}'>

<anchor id="SpeechBubble" entity={entity || null} at={at} offset={"0 " + height + " 0"} declutter="shift" occlusion="none" priority="200">
  <div id="Bubble" class:shout={shout} role="status">
    <div id="Box">
      <span id="Speaker" hidden={!speaker}>{speaker}</span>
      <p id="Line">{text}</p>
    </div>
    <svg id="Tail" viewBox="0 0 16 10"><path d="M0 0h16L8 10z"></path></svg>
  </div>
</anchor>

<style>
#SpeechBubble { align-items: center; }
#Bubble { align-items: center; animation: pop 200ms ease-out; }
#Box { max-width: 240px; padding: 8px 12px; gap: 2px; border-radius: 12px; background: #f4f1eaff; box-shadow: 0 3px 8px #00000066; }
.shout #Box { border: 2px solid var(--warn); }
#Speaker { font: var(--text-caption); color: #6b4a00ff; }
#Line { font: var(--text-sm); color: #1a1a1aff; }
#Tail { width: 16px; height: 10px; fill: #f4f1eaff; margin-top: -1px; }
@keyframes pop { from { scale: 0.6; opacity: 0; } to { scale: 1; opacity: 1; } }
</style>
