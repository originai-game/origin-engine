<meta name="component" content="RpgConversation">
<meta name="strings" content="./rpg.en.ogt">
<meta name="prop:speaker" content="string" data-doc="The speaker's name; empty is narration.">
<meta name="prop:portrait" content="number" data-doc="The speaker's portrait (og.load.texture); 0 hides it.">
<meta name="prop:line" content="string" data-doc="What is said now.">
<meta name="prop:replies" content="list" data-doc="The player's replies, keyed rows of { id, number, text, used }: number the figure before it; used greys a reply already chosen. Empty shows a Continue button.">
<meta name="prop:selected" content="string" data-doc="The reply the focus rests on; empty is none.">
<meta name="action:confirm" content="Continue">
<meta name="fixture:default" content='{"speaker":"Lady Marissa","portrait":0,"line":"The road ahead is dangerous.","replies":[{"id":"r1","number":1,"text":"We have no choice.","used":false},{"id":"r2","number":2,"text":"Perhaps another way.","used":true}],"selected":"r1"}'>
<meta name="fixture:line" content='{"speaker":"","portrait":0,"line":"The wind dies.","replies":[],"selected":""}'>

<!--
  A conversation across the foot of the screen: the speaker's portrait,
  name and line, and the player's numbered replies. Reports reply (a
  reply's key) and next (the Continue button, or confirm, while there are
  no replies).
-->
<section id="RpgConversation" class="rpg-conversation" aria-label={speaker ? speaker : t("rpg.talk.narration")}>
  <img class="portrait" src={portrait} alt="" hidden={!portrait}>
  <div class="said">
    <h3 class="speaker" hidden={!speaker}>{speaker}</h3>
    <p class="line">{line}</p>
  </div>
  {#if replies.length}
    <ol class="replies" role="listbox" aria-label={t("rpg.talk.replies")}>
      {#each replies as r (r.id)}
        <li class="reply-item"><button class="reply" role="option" data-used={r.used} aria-selected={r.id == selected} onclick={reply}><div class="face"><span class="reply-number">{t("rpg.talk.number", { number: r.number })}</span><span class="reply-text">{r.text}</span><div class="reply-mark"></div></div></button></li>
      {/each}
    </ol>
  {:else}
    <div class="go"><button id="Continue" class="secondary" onclick={next} onconfirm={next}>{t("rpg.talk.continue")}</button></div>
  {/if}
</section>

<style>
.rpg-conversation { display: flex; width: stretch; height: stretch; gap: var(--space-lg); container-type: inline-size; }
.portrait { width: calc(var(--size-xxl) * 1.9); height: stretch; object-fit: cover; object-position: center top; flex: none; }
.said { display: flex; flex-direction: column; width: 42%; gap: var(--space-sm); padding: var(--space-md) 0; flex: none; }
.speaker, .line { margin: 0; }
.replies { display: flex; flex-direction: column; margin: 0; padding: var(--space-md) 0; list-style: none; width: stretch; gap: var(--space-xxs); }
.reply-item { width: stretch; }
.reply { width: stretch; min-height: var(--size-touch); gap: var(--space-md); align-items: center; justify-content: flex-start; padding: var(--space-xs) var(--space-md); }
.reply .face { display: flex; width: stretch; align-items: center; gap: var(--space-md); }
.reply-number { flex: none; font-variant-numeric: tabular-nums; }
.reply-text { width: stretch; text-align: start; }
.reply-mark { width: var(--space-lg); height: var(--space-lg); flex: none; visibility: hidden; }
.reply[aria-selected=true] .reply-mark { visibility: visible; }
@media (pointer: fine) and (width >= 600px) { .reply { min-height: calc(var(--size-touch) * 0.8); } .replies { padding: var(--space-sm) 0; } }
.go { display: flex; flex-direction: column; width: stretch; align-items: flex-end; justify-content: flex-end; padding: var(--space-md); }
@container (width < 720px) {
  .rpg-conversation { flex-wrap: wrap; gap: var(--space-sm); }
  .portrait { width: var(--size-xxl); height: var(--size-xxl); }
  .said { width: stretch; flex: 1; padding: 0; }
  .replies { padding: 0; flex-basis: 100%; }
}
</style>
