<meta name="component" content="RpgConversation">
<meta name="strings" content="./conversation.en.ogt">
<meta name="prop:speaker" content="string" data-doc="The speaker's name; empty is narration.">
<meta name="prop:portrait" content="number" data-doc="The speaker's portrait (og.load.texture); 0 hides it.">
<meta name="prop:line" content="string" data-doc="What is said now.">
<meta name="prop:rate" content="number" data-default="0" data-doc="Characters of the line shown per second, starting again with each new line; 0 (and when left out) shows the line at once, and setting 0 finishes a line early.">
<meta name="prop:replies" content="list" data-doc="The player's replies, keyed rows of { id, number, text, used }: number the figure before it; used greys a reply already chosen. Empty shows a Continue button.">
<meta name="prop:selected" content="string" data-doc="The reply the focus rests on; empty is none.">
<meta name="prop:showControls" content="bool" data-default="false" data-doc="Shows optional History, Skip and Auto buttons; hidden in existing compact layouts.">
<meta name="prop:canBacklog" content="bool" data-default="true" data-doc="The game permits opening conversation history.">
<meta name="prop:canSkip" content="bool" data-default="false" data-doc="The game permits requesting a skip; the component never advances dialogue itself.">
<meta name="prop:canAuto" content="bool" data-default="true" data-doc="The game permits requesting an automatic-advance setting change.">
<meta name="prop:auto" content="bool" data-default="false" data-doc="The game's automatic-advance setting; toggleAuto requests a change without advancing dialogue.">
<meta name="action:confirm" content="Continue">
<meta name="fixture:default" content='{"speaker":"Lady Marissa","portrait":0,"line":"The road ahead is dangerous.","rate":0,"replies":[{"id":"r1","number":1,"text":"We have no choice.","used":false},{"id":"r2","number":2,"text":"Perhaps another way.","used":true}],"selected":"r1"}'>
<meta name="fixture:line" content='{"speaker":"","portrait":0,"line":"The wind dies.","replies":[],"selected":""}'>
<meta name="fixture:controls" content='{"speaker":"Marissa","line":"There is still time to turn back.","showControls":true,"canSkip":true,"replies":[]}'>
<meta name="fixture:auto" content='{"speaker":"","line":"The bell rings across the marsh.","showControls":true,"auto":true,"replies":[]}'>
<meta name="fixture:disabled" content='{"speaker":"","line":"Choose your answer before continuing.","showControls":true,"canBacklog":false,"canSkip":false,"canAuto":false,"replies":[]}'>

<!--
  A conversation across the foot of the screen: the speaker's portrait,
  name and line, and the player's numbered replies. Reports reply (a
  reply's key) and next (the Continue button, or confirm, while there are
  no replies). With a rate the line shows a few characters at a time
  (whole at once under reduced motion, and always whole to a screen
  reader) and reports revealed once it is whole; the game decides what a
  press before that does.
-->
<section class="frame" id="RpgConversation" part="frame" layout="stack" fill gap="wide" aria-label={speaker ? speaker : t("rpg.talk.narration")}>
  {#if portrait}<img class="portrait" src={portrait} alt="">{/if}
  <div class="said" layout="stack">
    {#if speaker}<h3 class="name" part="headline">{speaker}</h3>{/if}
    <og-typewriter id="Line" class="line" rate={rate} onrevealed={revealed}>{line}</og-typewriter>
    {#if showControls}<nav class="controls" part="controls" layout="wrap" gap="tight">
      <button part="backlog" disabled={!canBacklog} onclick={backlog}>{t("rpg.talk.backlog")}</button>
      <button part="skip" disabled={!canSkip} onclick={skip}>{t("rpg.talk.skip")}</button>
      <button class="auto" part="auto" aria-pressed={auto} disabled={!canAuto} onclick={toggleAuto} state:selected={auto}>{t("rpg.talk.auto")}</button>
    </nav>{/if}
  </div>
  {#if replies.length}
    <ol class="replies" layout="stack" gap="hair" fill role="listbox" aria-label={t("rpg.talk.replies")}>
      {#each replies as r (r.id)}
        <li class="reply-item"><button class="row" part="row" role="option" data-done={r.used} aria-selected={r.id == selected} onclick={reply} state:selected={r.id == selected} class:used={r.used}><div class="face" layout="row" gap="loose"><span part="figure" class="reply-number">{t("rpg.talk.number", { number: r.number })}</span><span class="reply-text" fill>{r.text}</span><div class="reply-mark"></div></div></button></li>
      {/each}
    </ol>
  {:else}
    <div class="go" layout="stack" fill><button id="Continue" class="secondary" onclick={next} onconfirm={next}>{t("rpg.talk.continue")}</button></div>
  {/if}
</section>

<style>
#RpgConversation { flex-flow: row wrap; overflow-y: auto; }
/* The band: cut glass lit gold along its top. */
.frame { background: var(--social-band-fill); box-shadow: var(--social-band-rim); border-radius: var(--quest-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.portrait { width: calc(var(--size-xxl) * 1.9); height: stretch; object-fit: cover; object-position: center top; flex: none; }
.said { width: 42%; padding: var(--space-md) 0; flex: none; }
/* The speaker on a gold plate, its far end cut. */
.name { margin: 0; align-self: flex-start; max-width: stretch; padding: var(--space-xxs) var(--space-lg) var(--space-xxs) var(--space-sm); overflow-wrap: anywhere; font-weight: var(--quest-weight); font-variant-caps: var(--alert-speaker-caps); letter-spacing: var(--hud-label-tracking); color: var(--quest-gold-ink); background: var(--social-nameplate-fill); clip-path: polygon(0 0, 100% 0, calc(100% - var(--quest-cut)) 100%, 0 100%); }
.line { display: block; margin: 0; font-size: var(--social-size-line); font-weight: var(--quest-weight); color: var(--ink-bright); }
.controls button { min-block-size: var(--size-touch); }
.auto:state(selected) { color: var(--quest-gold); box-shadow: var(--quest-ready-rim); }
.replies { margin: 0; padding: var(--space-md) 0; list-style: none; justify-content: center; }
.reply-item { width: stretch; }
/* A reply: a cut row, its key in a gold keyline; the one in focus washed
   gold with its key filled; a reply used grey with a faint key. */
.row { width: stretch; min-height: var(--size-touch); justify-content: flex-start; padding: var(--space-xs) var(--space-md); background: var(--social-line-fill); box-shadow: var(--quest-entry-rim); border-radius: var(--quest-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.row:hover { background: var(--quest-entry-fill); }
.row:state(selected) { background: var(--quest-selected-fill); box-shadow: var(--quest-selected-rim); }
.row .face { width: stretch; align-items: center; }
.reply-number { flex: none; min-inline-size: var(--size-sm); padding: 0 var(--space-xxs); text-align: center; font-weight: var(--hud-figure-weight); font-variant-numeric: var(--hud-figure-numeric); color: var(--quest-gold); background: var(--quest-chip-fill); box-shadow: var(--social-key-rim); }
.row:state(selected) .reply-number { color: var(--quest-gold-ink); background: var(--quest-gold); }
.row.used { color: var(--quest-ink-faint); }
.row.used .reply-number { color: var(--quest-ink-faint); }
.reply-text { text-align: start; overflow-wrap: anywhere; }
.reply-mark { width: var(--space-sm); height: var(--space-sm); flex: none; visibility: hidden; rotate: 45deg; background: var(--quest-gold); box-shadow: var(--quest-mark-lit); }
.row:state(selected) .reply-mark { visibility: visible; }
.row:not(:touch):not(:size(narrow)) { min-height: calc(var(--size-touch) * 0.8); }
.replies:not(:touch):not(:size(narrow)) { padding: var(--space-sm) 0; }
.go { align-items: flex-end; justify-content: flex-end; padding: var(--space-md); }
.portrait:size(compact) { width: var(--size-xxl); height: var(--size-xxl); }
.said:size(compact) { width: stretch; flex: 1; padding: 0; }
.line:size(compact) { font-size: var(--social-size-line-compact); }
.replies:size(compact), .go:size(compact) { padding: 0; flex-basis: 100%; }
</style>
