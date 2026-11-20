<meta name="component" content="RpgConversationBacklog">
<meta name="strings" content="./dialogue.en.ogt">
<meta name="action:back" content="Close history">
<meta name="prop:title" content="string" data-default="" data-doc="The history title; empty uses Conversation history.">
<meta name="prop:lines" content="list" data-default="[]" data-doc="Chronological keyed { id, speaker, text, canReplay } entries. Empty speaker is narration. The game supplies history and audio availability; lines keep their original wording.">
<meta name="prop:playing" content="string" data-default="" data-doc="The entry currently replaying; empty means no replay. The game owns playback and reports completion by clearing this id.">
<meta name="prop:hasOlder" content="bool" data-default="false" data-doc="Earlier history is available; older asks the game to prepend it, preserving row ids.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="History is loading; earlier-line and replay actions are disabled, close remains available.">
<meta name="fixture:default" content='{"title":"","lines":[{"id":"a","speaker":"Lady Marissa","text":"The road ahead is dangerous.","canReplay":true},{"id":"b","speaker":"Brenna","text":"We have no choice.","canReplay":false},{"id":"c","speaker":"","text":"The wind dies.","canReplay":false}],"playing":"","hasOlder":true,"loading":false}'>
<meta name="fixture:empty" content='{}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:playing" content='{"playing":"a","lines":[{"id":"a","speaker":"Lady Marissa","text":"The road ahead is dangerous.","canReplay":true}]}'>

<section id="RpgConversationBacklog" part="frame" layout="stack" gap="loose" aria-label={title || t("rpg.talk.history")} aria-busy={loading} onback={close}>
  <header class="header" part="header" layout="wrap"><h2 class="title" part="title" fill>{title || t("rpg.talk.history")}</h2><button part="close" onclick={close}>{t("rpg.talk.close")}</button></header>
  {#if loading}<div class="status" part="caption" layout="row" role="status"><Spinner name={t("rpg.talk.historyLoading")} /><span>{t("rpg.talk.historyLoading")}</span></div>{/if}
  <div part="lines" layout="scroll" gap="loose" fill role="log" aria-label={t("rpg.talk.history")}>
    {#if hasOlder}<button class="older" part="older" disabled={loading} onclick={older}>{t("rpg.talk.older")}</button>{/if}
    {#each lines as entry (entry.id)}<article class="row" part="row" layout="stack" data-selected={entry.id == playing} class:playing={entry.id == playing}>
      <h3 class="speaker" part="headline" class:narration={!entry.speaker}>{entry.speaker || t("rpg.talk.narration")}</h3>
      <p class="text" part="text" class:narration={!entry.speaker}>{entry.text}</p>
      {#if entry.id == playing}<button class="stop" part="stop" onclick={stop}>{t("rpg.talk.stop")}</button>
      {:else if entry.canReplay}<button class="replay" part="replay" disabled={loading} onclick={replay}>{t("rpg.talk.replay", { speaker: entry.speaker || t("rpg.talk.narration") })}</button>{/if}
    </article>{/each}
    {#if !lines.length && !loading}<p class="empty" part="caption" role="status">{t("rpg.talk.historyEmpty")}</p>{/if}
  </div>
</section>

<style>
#RpgConversationBacklog { inline-size: stretch; block-size: stretch; }
h2, h3, p { margin: 0; }
.header { flex: none; }
.title { flex: 1 1 auto; }
/* A line: a cut card; the speaker in gold small capitals, narration
   quiet and in italics; the line replaying washed gold with a gold bar. */
.row { flex: none; padding: var(--space-sm) var(--space-md); background: var(--quest-entry-fill); box-shadow: var(--quest-entry-rim); border-radius: var(--quest-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.row.playing { background: var(--quest-ready-fill); box-shadow: var(--quest-ready-rim); }
.speaker { overflow-wrap: anywhere; font: var(--caption-text); font-weight: var(--quest-weight); font-variant-caps: var(--alert-speaker-caps); letter-spacing: var(--hud-label-tracking); color: var(--quest-gold); }
.speaker.narration { color: var(--quest-ink-quiet); }
.text { overflow-wrap: anywhere; color: var(--ink-bright); }
.text.narration { font-style: var(--social-narration-style); color: var(--quest-ink-quiet); }
button { min-block-size: var(--size-touch); }
.replay, .stop { align-self: flex-start; }
.older { flex: none; align-self: center; }
.empty { padding: var(--space-lg); text-align: center; color: var(--quest-ink-quiet); }
</style>
