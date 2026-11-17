<meta name="component" content="Inbox">
<meta name="strings" content="./meta.en.ogt">
<meta name="parts" content="Inbox.frame Inbox.band Inbox.title Inbox.summary Inbox.layout Inbox.list Inbox.row Inbox.sender Inbox.subject Inbox.date Inbox.preview Inbox.badge Inbox.detail Inbox.body Inbox.attachments Inbox.attachment Inbox.icon Inbox.name Inbox.amount Inbox.state Inbox.actions Inbox.claim Inbox.delete Inbox.empty Inbox.caption Inbox.headline Inbox.figure">
<meta name="prop:title" content="string" data-default="" data-doc="Heading; empty uses Inbox.">
<meta name="prop:summary" content="string" data-default="" data-doc="The game's count or capacity wording, such as 2 unread · 8 / 50.">
<meta name="prop:messages" content="list" data-default="[]" data-doc="Keyed { id, subject, sender, date, preview, body, unread, hasAttachments, claimable, claimed, expired, canDelete } messages. Text is plain, not HTML. The game updates read/claim/delete facts after intents.">
<meta name="prop:attachments" content="list" data-default="[]" data-doc="Keyed { id, message, name, icon, amount } rewards; message is the owning message id.">
<meta name="prop:selected" content="string" data-default="" data-doc="Selected message id, owned by the game; empty shows a choose-message prompt.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Loading messages; no message actions are available.">
<meta name="prop:busy" content="bool" data-default="false" data-doc="A claim/delete operation is pending; prevent duplicate mutations.">
<meta name="prop:error" content="string" data-default="" data-doc="Loading failed; show recovery advice and retry.">
<meta name="action:choose" content="Read the message by stable key">
<meta name="action:claim" content="Claim the selected message's attachments by stable message key">
<meta name="action:delete" content="Request deleting the selected message by stable key">
<meta name="action:retry" content="Retry loading messages">
<meta name="fixture:default" content='{"summary":"1 unread · 2 / 50","selected":"welcome","messages":[{"id":"welcome","subject":"Welcome to the expedition","sender":"Expedition Guild","date":"16 Nov","preview":"Supplies for your first journey.","body":"Welcome, adventurer. These supplies will help you reach the northern camp.\n\nGood luck on your first expedition.","unread":true,"hasAttachments":true,"claimable":true,"claimed":false,"expired":false,"canDelete":true},{"id":"notice","subject":"Scheduled maintenance","sender":"Game team","date":"15 Nov","preview":"Service resumes at 09:00 UTC.","body":"The world will be unavailable during maintenance. All progress is safe.","unread":false,"hasAttachments":false,"claimable":false,"claimed":false,"expired":false,"canDelete":true}],"attachments":[{"id":"gold","message":"welcome","name":"Gold","icon":"coin","amount":"500"},{"id":"potions","message":"welcome","name":"Health potions","icon":"potion","amount":"3"}]}'>
<meta name="fixture:empty" content='{}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:error" content='{"error":"Messages could not be loaded. Reconnect and try again."}'>
<meta name="fixture:expired" content='{"selected":"old","messages":[{"id":"old","subject":"Rewards from the previous season","sender":"Season team","date":"1 Oct","preview":"The claim window has closed.","body":"These attachments have expired. See you next season.","unread":false,"hasAttachments":true,"claimable":false,"claimed":false,"expired":true,"canDelete":true}]}'>

<section id="Inbox" part="Inbox.frame" data-layout="shell" data-gap="loose" aria-label={title || t("meta.inbox.title")} aria-busy={loading || busy}>
  <header part="Inbox.band" class="band" data-layout="cluster"><h2 part="Inbox.title" data-fill>{title || t("meta.inbox.title")}</h2>{#if summary}<span part="Inbox.summary Inbox.caption" class="muted">{summary}</span>{/if}</header>
  {#if loading}<div class="wait" data-layout="row" role="status"><Spinner name={t("meta.loading")} /><span>{t("meta.loading")}</span></div>
  {:else if error}<ErrorState body={error} />
  {:else if !messages.length}<EmptyState title={t("meta.inbox.empty")} icon="bag" />
  {:else}<div part="Inbox.layout" class="layout" data-layout="grid" data-gap="loose">
    <ul part="Inbox.list" class="list" data-layout="scroll" data-gap="tight" aria-label={t("meta.inbox.messages")}>
      {#each messages as message (message.id)}<li><button part="Inbox.row" class="row" aria-pressed={message.id == selected} data-selected={message.id == selected} data-unread={message.unread} disabled={busy} onclick={choose}><div class="row-copy" data-layout="stack">
        <span part="Inbox.sender Inbox.caption" class="muted">{message.sender}</span><span part="Inbox.subject Inbox.headline" class="subject" title={message.subject}>{message.subject}</span><span part="Inbox.preview Inbox.caption" class="muted">{message.preview}</span><div class="row-meta" data-layout="cluster"><span part="Inbox.date Inbox.caption" class="muted">{message.date}</span><span part="Inbox.badge" class="badge">{message.unread ? t("meta.inbox.unread") : t("meta.inbox.read")}</span>{#if message.hasAttachments}<og-icon part="Inbox.icon" class="icon" icon="bag"></og-icon>{/if}</div>
      </div></button></li>{/each}
    </ul>
    <div class="reader">
      {#if !selected}<p part="Inbox.empty Inbox.caption" class="muted">{t("meta.inbox.select")}</p>{/if}
      {#each messages as message (message.id)}{#if message.id == selected}<article part="Inbox.detail" class="detail" data-layout="stack" aria-label={message.subject}>
        <h3 part="Inbox.subject Inbox.headline">{message.subject}</h3><div class="row-meta" data-layout="cluster"><span part="Inbox.sender Inbox.caption" class="muted">{message.sender}</span><span part="Inbox.date Inbox.caption" class="muted">{message.date}</span></div><p part="Inbox.body" class="body">{message.body}</p>
        {#if message.hasAttachments}<section part="Inbox.attachments" class="attachments" data-layout="stack" data-gap="tight" aria-label={t("meta.inbox.attachments")}><h4>{t("meta.inbox.attachments")}</h4><ul class="attachment-list" data-layout="stack" data-gap="tight">
          {#each attachments as attachment (attachment.id)}{#if attachment.message == message.id}<li part="Inbox.attachment" class="attachment" data-layout="cluster"><og-icon part="Inbox.icon" class="icon" icon={attachment.icon}></og-icon><div part="Inbox.name">{attachment.name}</div><div part="Inbox.amount Inbox.figure" class="amount">{attachment.amount}</div></li>{/if}{/each}
        </ul><span part="Inbox.state Inbox.caption" class="muted" data-state={message.expired ? 'expired' : message.claimed ? 'claimed' : 'available'}>{message.expired ? t("meta.inbox.expired") : message.claimed ? t("meta.claimed") : t("meta.available")}</span></section>{/if}
        <footer part="Inbox.actions" class="actions" data-layout="cluster">
          {#if message.hasAttachments}<button id="Claim" part="Inbox.claim" variant="primary" disabled={busy || !message.claimable || message.claimed || message.expired} onclick={claim}>{t("meta.inbox.claim")}</button>{/if}
          {#if message.canDelete}<button id="Delete" part="Inbox.delete" disabled={busy} onclick={delete}>{t("meta.inbox.delete")}</button>{/if}
        </footer>
      </article>{/if}{/each}
    </div>
  </div>{/if}
</section>

<style>
:host { block-size: auto; }
h2, h3, h4, p { margin: 0; overflow-wrap: anywhere; }
.muted { overflow-wrap: anywhere; }
.layout { grid-template-columns: minmax(0, 1fr) minmax(0, 1.5fr); }
.list { max-block-size: var(--measure-xl); margin: 0; padding: var(--space-xxs); list-style: none; }
.row { display: block; inline-size: stretch; padding: var(--space-sm); text-align: start; }
.row-copy, .detail { align-items: start; }
.row-copy { inline-size: stretch; }
.subject { inline-size: stretch; overflow-wrap: anywhere; line-clamp: 1; text-overflow: ellipsis; }
.icon { inline-size: var(--size-xs); block-size: var(--size-xs); flex: none; }
.reader { min-inline-size: 0; max-block-size: var(--measure-xl); overflow-y: auto; }
.body { white-space: pre-wrap; overflow-wrap: anywhere; }
.attachments { inline-size: stretch; }
.attachment-list { margin: 0; padding: 0; list-style: none; }
.amount { margin-inline-start: auto; }
.actions button { min-block-size: var(--touch-target); white-space: normal; }
.wait { padding: var(--space-lg); }
@container (width < 600px) { .layout { grid-template-columns: minmax(0, 1fr); } .list { max-block-size: var(--size-5xl); } .reader { max-block-size: none; overflow: visible; } }
</style>
