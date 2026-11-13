<meta name="component" content="RpgConversation">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgConversation.frame RpgConversation.name RpgConversation.row">
<meta name="prop:speaker" content="string" data-doc="The speaker's name; empty is narration.">
<meta name="prop:portrait" content="number" data-doc="The speaker's portrait (og.load.texture); 0 hides it.">
<meta name="prop:line" content="string" data-doc="What is said now.">
<meta name="prop:rate" content="number" data-default="0" data-doc="Characters of the line shown per second, starting again with each new line; 0 (and when left out) shows the line at once, and setting 0 finishes a line early.">
<meta name="prop:replies" content="list" data-doc="The player's replies, keyed rows of { id, number, text, used }: number the figure before it; used greys a reply already chosen. Empty shows a Continue button.">
<meta name="prop:selected" content="string" data-doc="The reply the focus rests on; empty is none.">
<meta name="action:confirm" content="Continue">
<meta name="fixture:default" content='{"speaker":"Lady Marissa","portrait":0,"line":"The road ahead is dangerous.","rate":0,"replies":[{"id":"r1","number":1,"text":"We have no choice.","used":false},{"id":"r2","number":2,"text":"Perhaps another way.","used":true}],"selected":"r1"}'>
<meta name="fixture:line" content='{"speaker":"","portrait":0,"line":"The wind dies.","replies":[],"selected":""}'>

<!--
  A conversation across the foot of the screen: the speaker's portrait,
  name and line, and the player's numbered replies. Reports reply (a
  reply's key) and next (the Continue button, or confirm, while there are
  no replies). With a rate the line shows a few characters at a time
  (whole at once under reduced motion, and always whole to a screen
  reader) and reports revealed once it is whole; the game decides what a
  press before that does.
-->
<section id="RpgConversation" part="RpgConversation.frame" aria-label={speaker ? speaker : t("rpg.talk.narration")}>
  <img class="portrait" src={portrait} alt="" hidden={!portrait}>
  <div class="said">
    <h3 part="RpgConversation.name" hidden={!speaker}>{speaker}</h3>
    <og-typewriter id="Line" class="line" rate={rate} onrevealed={revealed}>{line}</og-typewriter>
  </div>
  {#if replies.length}
    <ol class="replies" role="listbox" aria-label={t("rpg.talk.replies")}>
      {#each replies as r (r.id)}
        <li class="reply-item"><button part="RpgConversation.row" role="option" data-used={r.used} aria-selected={r.id == selected} onclick={reply}><div class="face"><span class="reply-number">{t("rpg.talk.number", { number: r.number })}</span><span class="reply-text">{r.text}</span><div class="reply-mark"></div></div></button></li>
      {/each}
    </ol>
  {:else}
    <div class="go"><button id="Continue" class="secondary" onclick={next} onconfirm={next}>{t("rpg.talk.continue")}</button></div>
  {/if}
</section>

<style>
[part="RpgConversation.frame"][hidden], [part="RpgConversation.frame"] [hidden] { display: none; }
[part="RpgConversation.frame"] { display: flex; width: stretch; height: stretch; gap: var(--space-lg); container-type: inline-size; }
.portrait { width: calc(var(--size-xxl) * 1.9); height: stretch; object-fit: cover; object-position: center top; flex: none; }
.said { display: flex; flex-direction: column; width: 42%; gap: var(--space-sm); padding: var(--space-md) 0; flex: none; }
[part="RpgConversation.name"], .line { margin: 0; }
.line { display: block; }
.replies { display: flex; flex-direction: column; margin: 0; padding: var(--space-md) 0; list-style: none; width: stretch; gap: var(--space-xxs); }
.reply-item { width: stretch; }
[part="RpgConversation.row"] { width: stretch; min-height: var(--size-touch); gap: var(--space-md); align-items: center; justify-content: flex-start; padding: var(--space-xs) var(--space-md); }
[part="RpgConversation.row"] .face { display: flex; width: stretch; align-items: center; gap: var(--space-md); }
.reply-number { flex: none; font-variant-numeric: tabular-nums; }
.reply-text { width: stretch; text-align: start; }
.reply-mark { width: var(--space-lg); height: var(--space-lg); flex: none; visibility: hidden; }
[part="RpgConversation.row"][aria-selected=true] .reply-mark { visibility: visible; }
@media (pointer: fine) and (width >= 600px) { [part="RpgConversation.row"] { min-height: calc(var(--size-touch) * 0.8); } .replies { padding: var(--space-sm) 0; } }
.go { display: flex; flex-direction: column; width: stretch; align-items: flex-end; justify-content: flex-end; padding: var(--space-md); }
@container (width < 720px) {
  [part="RpgConversation.frame"] { flex-wrap: wrap; gap: var(--space-sm); }
  .portrait { width: var(--size-xxl); height: var(--size-xxl); }
  .said { width: stretch; flex: 1; padding: 0; }
  .replies { padding: 0; flex-basis: 100%; }
}
</style>
