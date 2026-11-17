<meta name="component" content="News">
<meta name="strings" content="./meta.en.ogt">
<meta name="parts" content="News.frame News.band News.title News.list News.article News.card News.heading News.toggle News.headline News.date News.kind News.caption News.badge News.picture News.summary News.body News.action">
<meta name="prop:title" content="string" data-default="" data-doc="Heading; empty uses News and updates.">
<meta name="prop:entries" content="list" data-default="[]" data-doc="Keyed { id, title, date, kind, summary, body, image, unread, pinned, actionLabel } articles in the game's order. kind is game-formatted text (Patch notes, Event); body is plain multiline text, never HTML. image is a loaded texture handle or 0.">
<meta name="prop:selected" content="string" data-default="" data-doc="Expanded article id, game-owned. A toggle reports choose with that key; the game may clear it to collapse.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Loading articles; no article actions are present.">
<meta name="prop:disabled" content="bool" data-default="false" data-doc="Disable article toggles and actions.">
<meta name="prop:error" content="string" data-default="" data-doc="Failure explanation with retry; empty is no error.">
<meta name="action:choose" content="Toggle an article by stable key">
<meta name="action:open" content="Take the selected article's optional action by stable key">
<meta name="action:retry" content="Retry loading articles">
<meta name="fixture:default" content='{"selected":"patch","entries":[{"id":"patch","title":"Expedition update 1.4","date":"16 Nov 2026","kind":"Patch notes","summary":"New destinations, equipment balance and accessibility improvements.","body":"New destinations\nExplore the northern coast with your party.\n\nAccessibility\nController navigation now remembers the last focused item.\n\nBalance\nHeavy weapons recover faster after a missed attack.","image":0,"unread":true,"pinned":true,"actionLabel":"View full release notes"},{"id":"event","title":"Double rewards weekend","date":"20–22 Nov","kind":"Event","summary":"Earn extra expedition rewards all weekend.","body":"All expedition rewards are doubled during the event.","image":0,"unread":false,"pinned":false,"actionLabel":""}]}'>
<meta name="fixture:empty" content='{}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:error" content='{"error":"Updates are temporarily unavailable. Try again when connected."}'>
<meta name="fixture:overflow" content='{"selected":"long","entries":[{"id":"long","title":"The northern expedition and the return of the Watcher of the Ninefold Gate","date":"16 November 2026 · 09:00 UTC","kind":"Community event and balance notes","summary":"An announcement with a long title and detailed changes that remain readable at phone widths.","body":"Expedition changes\nThe new route includes additional checkpoints and improved navigation markers.\n\nEquipment changes\nLong equipment descriptions wrap instead of hiding important information.","image":0,"unread":true,"pinned":true,"actionLabel":"Read the complete expedition update"}]}'>

<section id="News" part="News.frame" data-layout="stack" data-gap="loose" aria-label={title || t("meta.news.title")} aria-busy={loading}>
  <header part="News.band"><h2 part="News.title">{title || t("meta.news.title")}</h2></header>
  {#if loading}<div class="wait" data-layout="row" role="status"><Spinner name={t("meta.loading")} /><span>{t("meta.loading")}</span></div>
  {:else if error}<ErrorState body={error} />
  {:else if !entries.length}<EmptyState title={t("meta.news.empty")} icon="flag" />
  {:else}<ul part="News.list" class="list" data-layout="stack" data-gap="loose">
    {#each entries as entry (entry.id)}<li><article part="News.article News.card" class="article" data-layout="stack" data-unread={entry.unread} data-pinned={entry.pinned}>
      <div class="meta" data-layout="cluster"><span part="News.kind News.caption" class="kind">{entry.kind}</span><span part="News.date News.caption" class="date">{entry.date}</span>{#if entry.pinned}<span part="News.badge" class="badge">{t("meta.news.pinned")}</span>{/if}{#if entry.unread}<span part="News.badge" class="badge">{t("meta.news.unread")}</span>{/if}</div>
      <div part="News.heading" role="heading" aria-level="3"><button part="News.toggle" class="toggle" aria-expanded={entry.id == selected} aria-label={entry.title} disabled={disabled} onclick={choose}><span part="News.headline" class="headline">{entry.title}</span></button></div>
      {#if entry.summary}<p part="News.summary News.caption" class="summary">{entry.summary}</p>{/if}
      {#if entry.id == selected}
        {#if entry.image}<img part="News.picture" class="picture" src={entry.image} alt="">{/if}
        {#if entry.body}<p part="News.body" class="body">{entry.body}</p>{/if}
        {#if entry.actionLabel}<button part="News.action" class="action" disabled={disabled} onclick={open}>{entry.actionLabel}</button>{/if}
      {/if}
    </article></li>{/each}
  </ul>{/if}
</section>

<style>
:host { inline-size: stretch; }
h2, h3, p { margin: 0; overflow-wrap: anywhere; }
.list { margin: 0; padding: 0; list-style: none; }
.article { align-items: start; }
[part="News.heading"] { inline-size: stretch; }
.toggle { display: block; inline-size: stretch; min-block-size: var(--touch-target); text-align: start; white-space: normal; overflow-wrap: anywhere; }
.headline { display: block; inline-size: stretch; white-space: normal; overflow-wrap: anywhere; }
.picture { inline-size: 100%; max-block-size: var(--size-6xl); aspect-ratio: 2; object-fit: cover; }
.body { white-space: pre-wrap; }
.action { max-inline-size: 100%; min-block-size: var(--touch-target); white-space: normal; }
.wait { padding: var(--space-lg); }
</style>
