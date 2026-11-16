<meta name="component" content="RpgConversationBacklog">
<meta name="strings" content="./dialogue.en.ogt">
<meta name="action:back" content="Close history">
<meta name="parts" content="RpgConversationBacklog.frame RpgConversationBacklog.header RpgConversationBacklog.title RpgConversationBacklog.close RpgConversationBacklog.lines RpgConversationBacklog.row RpgConversationBacklog.speaker RpgConversationBacklog.text RpgConversationBacklog.replay RpgConversationBacklog.stop RpgConversationBacklog.older RpgConversationBacklog.status RpgConversationBacklog.empty">
<meta name="prop:title" content="string" data-default="" data-doc="The history title; empty uses Conversation history.">
<meta name="prop:lines" content="list" data-default="[]" data-doc="Chronological keyed { id, speaker, text, canReplay } entries. Empty speaker is narration. The game supplies history and audio availability; lines keep their original wording.">
<meta name="prop:playing" content="string" data-default="" data-doc="The entry currently replaying; empty means no replay. The game owns playback and reports completion by clearing this id.">
<meta name="prop:hasOlder" content="bool" data-default="false" data-doc="Earlier history is available; older asks the game to prepend it, preserving row ids.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="History is loading; earlier-line and replay actions are disabled, close remains available.">
<meta name="fixture:default" content='{"title":"","lines":[{"id":"a","speaker":"Lady Marissa","text":"The road ahead is dangerous.","canReplay":true},{"id":"b","speaker":"Brenna","text":"We have no choice.","canReplay":false},{"id":"c","speaker":"","text":"The wind dies.","canReplay":false}],"playing":"","hasOlder":true,"loading":false}'>
<meta name="fixture:empty" content='{}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:playing" content='{"playing":"a","lines":[{"id":"a","speaker":"Lady Marissa","text":"The road ahead is dangerous.","canReplay":true}]}'>

<section id="RpgConversationBacklog" part="RpgConversationBacklog.frame" aria-label={title || t("rpg.talk.history")} aria-busy={loading} onback={close}>
  <header part="RpgConversationBacklog.header"><h2 part="RpgConversationBacklog.title">{title || t("rpg.talk.history")}</h2><button part="RpgConversationBacklog.close" onclick={close}>{t("rpg.talk.close")}</button></header>
  {#if loading}<div part="RpgConversationBacklog.status" role="status"><Spinner name={t("rpg.talk.historyLoading")} /><span>{t("rpg.talk.historyLoading")}</span></div>{/if}
  <div part="RpgConversationBacklog.lines" role="log" aria-label={t("rpg.talk.history")}>
    {#if hasOlder}<button part="RpgConversationBacklog.older" disabled={loading} onclick={older}>{t("rpg.talk.older")}</button>{/if}
    {#each lines as entry (entry.id)}<article part="RpgConversationBacklog.row" data-playing={entry.id == playing}>
      <h3 part="RpgConversationBacklog.speaker">{entry.speaker || t("rpg.talk.narration")}</h3>
      <p part="RpgConversationBacklog.text">{entry.text}</p>
      {#if entry.id == playing}<button part="RpgConversationBacklog.stop" onclick={stop}>{t("rpg.talk.stop")}</button>
      {:else if entry.canReplay}<button part="RpgConversationBacklog.replay" disabled={loading} onclick={replay}>{t("rpg.talk.replay", { speaker: entry.speaker || t("rpg.talk.narration") })}</button>{/if}
    </article>{/each}
    {#if !lines.length && !loading}<p part="RpgConversationBacklog.empty" role="status">{t("rpg.talk.historyEmpty")}</p>{/if}
  </div>
</section>

<style>
:host { display: flex; flex-direction: column; inline-size: stretch; block-size: stretch; gap: var(--space-md); }
h2, h3, p { margin: 0; }
[part="RpgConversationBacklog.header"] { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: var(--space-sm); flex: none; }
[part="RpgConversationBacklog.title"] { font-size: var(--font-step-lg); }
[part="RpgConversationBacklog.lines"] { display: flex; flex-direction: column; flex: 1; min-block-size: 0; overflow-y: auto; gap: var(--space-md); }
[part="RpgConversationBacklog.row"] { display: flex; flex-direction: column; flex: none; gap: var(--space-sm); padding: var(--space-md); border-inline-start: var(--stroke-regular) solid var(--line); }
[part="RpgConversationBacklog.row"][data-playing=true] { background: var(--selection); border-color: var(--focus); }
[part="RpgConversationBacklog.speaker"] { font: var(--text-caption); font-weight: var(--font-weight-semibold); }
[part="RpgConversationBacklog.text"] { line-height: var(--font-leading-normal); }
button { min-block-size: var(--size-touch); }
[part="RpgConversationBacklog.replay"], [part="RpgConversationBacklog.stop"] { align-self: flex-start; }
[part="RpgConversationBacklog.status"], [part="RpgConversationBacklog.empty"] { color: var(--ink-muted); }
[part="RpgConversationBacklog.status"] { display: flex; align-items: center; gap: var(--space-sm); }
</style>
