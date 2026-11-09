<meta name="component" content="RpgDialogue">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="Dialogue.frame Dialogue.speaker Dialogue.choice">
<meta name="prop:speaker" content="string" data-doc="Who speaks; empty for narration.">
<meta name="prop:epithet" content="string" data-doc="The speaker's line under the name (Ferry warden); empty hides it.">
<meta name="prop:portrait" content="number" data-doc="A texture for the speaker (og.load.texture); 0 draws no portrait.">
<meta name="prop:line" content="string" data-doc="What is said now, as the game reveals it.">
<meta name="prop:choices" content="list" data-doc="The replies in the game's order: each { id, key, text, check, seen, locked, ends }. key is the reply's number or shortcut as the game shows it (1, 2); check names a roll or requirement as the game words it (Persuasion · DC 14), empty for none; seen greys a reply already taken; locked one the game refuses (it shows, disabled); ends marks a reply that leaves.">
<meta name="prop:selected" content="string" data-doc="The id of the highlighted reply; empty is none.">
<meta name="prop:canContinue" content="bool" data-doc="With no replies, Continue is offered.">
<meta name="prop:canLog" content="bool" data-doc="The log of earlier lines is offered.">
<meta name="action:confirm" content="Continue">
<meta name="action:details" content="Log">
<meta name="fixture:default" content='{"speaker":"Old Ysolde","epithet":"Ferry warden","portrait":0,"line":"He rowed out with the tithe and never came back. Light the beacons, and the water gives up what it keeps.","choices":[{"id":"c1","key":"1","text":"What did he owe, and to whom?","check":"","seen":false,"locked":false,"ends":false},{"id":"c2","key":"2","text":"You know more than you are telling me.","check":"Insight · DC 12","seen":false,"locked":false,"ends":false},{"id":"c3","key":"3","text":"Pay me first.","check":"Requires 50 gold","seen":false,"locked":true,"ends":false},{"id":"c4","key":"4","text":"I will see to it.","check":"","seen":true,"locked":false,"ends":true}],"selected":"c2","canContinue":false,"canLog":true}'>
<meta name="fixture:narration" content='{"speaker":"","epithet":"","portrait":0,"line":"The bell tolls once under the water.","choices":[],"selected":"","canContinue":true,"canLog":false}'>

<!--
  The rpg kit's conversation (README.md): the speaker's portrait, name and
  line, and the replies as numbered rows, a check or requirement beside
  each. The game owns the tree, the rolls and what a reply costs; the
  dialogue reports choose (the reply's key), advance and log. It sits at
  the bottom of the screen and keeps every reply reachable on a phone.
-->
<section id="RpgDialogue" part="Dialogue.frame" aria-label={speaker || t("rpg.dialogue.narration")}>
  <div id="Speech">
    <div id="Portrait" part="Dialogue.speaker" hidden={!portrait}><img id="Face" src={portrait} alt=""></div>
    <div id="Said">
      {#if speaker}
      <div id="Who">
        <h3 id="Speaker">{speaker}</h3>
        <span id="Epithet" hidden={!epithet}>{epithet}</span>
      </div>
      {/if}
      <p id="Line" aria-live="polite">{line}</p>
    </div>
  </div>
  {#if choices.length}
    <div id="Replies" role="listbox" aria-label={t("rpg.dialogue.replies")}>
      {#each choices as c (c.id)}
        <button id="Reply" part="Dialogue.choice" class="reply" class:seen={c.seen} class:ends={c.ends} class:checked={!!c.check} variant="quiet" role="option" aria-selected={c.id == selected} aria-description={c.check} disabled={c.locked} onclick={choose}>
          <span class="index number">{c.key}</span>
          <div class="said">
            <span class="text">{c.text}</span>
            <span class="check" hidden={!c.check}>{c.check}</span>
          </div>
        </button>
      {/each}
    </div>
  {/if}
  <div id="Actions">
    <button id="Log" variant="ghost" hidden={!canLog} onclick={log} ondetails={log}><kbd id="LogKey" data-action="details"></kbd><span>{t("rpg.dialogue.log")}</span></button>
    {#if !choices.length && canContinue}
      <button id="Continue" variant="primary" onclick={advance} onconfirm={advance}><kbd id="ContinueKey" data-action="confirm"></kbd><span>{t("rpg.dialogue.continue")}</span></button>
    {/if}
  </div>
</section>

<style>
#RpgDialogue { width: stretch; gap: var(--space-md); padding: var(--space-lg); background: var(--panel); border-radius: var(--radius-lg); container-type: inline-size; }
#Speech { display: flex; width: stretch; gap: var(--space-lg); align-items: start; }
#Portrait { width: calc(5 * var(--space-lg)); aspect-ratio: 1; flex-shrink: 0; overflow: hidden; background: var(--panel-raised); border-radius: var(--radius-md); padding: var(--space-xxs); }
#Face { width: stretch; height: stretch; object-fit: cover; }
#Said { width: stretch; gap: var(--space-xs); min-width: 0; }
#Who { display: flex; flex-wrap: wrap; gap: var(--space-xs) var(--space-sm); align-items: baseline; }
#Speaker { margin: 0; font: var(--text-lg); font-family: fantasy; color: var(--highlight); }
#Epithet { font: var(--text-sm); color: var(--ink-muted); font-style: italic; }
#Line { margin: 0; width: stretch; font: var(--text-reading); color: var(--ink); }
.number { font-family: monospace; font-variant-numeric: tabular-nums; white-space: nowrap; }
#Replies { width: stretch; gap: var(--space-xxs); border-top: var(--stroke-hairline) solid var(--line); padding-top: var(--space-sm); }
.reply { width: stretch; min-height: var(--size-touch); padding: var(--space-xs) var(--space-md); gap: var(--space-sm); align-items: center; justify-content: start; text-align: start; }
.index { width: var(--space-lg); flex-shrink: 0; color: var(--ink-faint); }
.said { width: stretch; gap: 0; min-width: 0; }
.text { width: stretch; color: var(--ink); }
.reply.seen .text { color: var(--ink-muted); }
.reply.ends .text { font-style: italic; }
.reply[aria-selected=true] .index { color: var(--highlight); }
.check { font: var(--text-caption); color: var(--accent); }
.reply:disabled .check { color: var(--ink-faint); }
#Actions { display: flex; gap: var(--space-sm); width: stretch; justify-content: end; }
#Actions button { min-height: var(--size-touch); }
@container (width < 560px) {
  #RpgDialogue { padding: var(--space-md); }
  #Portrait { width: calc(3 * var(--space-lg)); }
  #Speech { gap: var(--space-md); }
}
@media (pointer: coarse) { kbd { display: none; } }
</style>
