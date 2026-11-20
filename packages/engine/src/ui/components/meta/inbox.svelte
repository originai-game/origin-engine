<meta name="component" content="Inbox">
<meta name="strings" content="./meta.en.ogt">
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

<section id="Inbox" part="frame" layout="stack" fill gap="loose" aria-label={title || t("meta.inbox.title")} aria-busy={loading || busy}>
  <header part="band" class="band" layout="wrap"><h2 part="title" fill>{title || t("meta.inbox.title")}</h2>{#if summary}<span part="caption" class="muted summary">{summary}</span>{/if}</header>
  {#if loading}<div class="wait" layout="row" role="status"><Spinner name={t("meta.loading")} /><span>{t("meta.loading")}</span></div>
  {:else if error}<ErrorState body={error} />
  {:else if !messages.length}<EmptyState title={t("meta.inbox.empty")} icon="bag" />
  {:else}<div part="layout" class="layout grid" gap="loose">
    <ul part="list" class="list" layout="scroll" gap="tight" aria-label={t("meta.inbox.messages")}>
      {#each messages as message (message.id)}<li><button part="row" class="row" aria-pressed={message.id == selected} disabled={busy} onclick={choose} class:not-unread={!(message.unread)} class:unread={message.unread} state:selected={message.id == selected}><div class="row-copy" layout="stack">
        <span part="caption" class="muted sender">{message.sender}</span><span part="headline" class="subject" title={message.subject}>{message.subject}</span><span part="caption" class="muted preview">{message.preview}</span><div class="row-meta" layout="wrap"><span part="caption" class="muted date">{message.date}</span><span part="badge" class="badge">{message.unread ? t("meta.inbox.unread") : t("meta.inbox.read")}</span>{#if message.hasAttachments}<og-icon part="icon" class="icon" icon="bag"></og-icon>{/if}</div>
      </div></button></li>{/each}
    </ul>
    <div class="reader">
      {#if !selected}<p part="caption" class="muted empty">{t("meta.inbox.select")}</p>{/if}
      {#each messages as message (message.id)}{#if message.id == selected}<article part="detail" class="detail" layout="stack" aria-label={message.subject}>
        <h3 class="subject" part="headline">{message.subject}</h3><div class="row-meta" layout="wrap"><span part="caption" class="muted sender">{message.sender}</span><span part="caption" class="muted date">{message.date}</span></div><p part="body" class="body">{message.body}</p>
        {#if message.hasAttachments}<section part="attachments" class="attachments" layout="stack" gap="tight" aria-label={t("meta.inbox.attachments")}><h4>{t("meta.inbox.attachments")}</h4><ul class="attachment-list" layout="stack" gap="tight">
          {#each attachments as attachment (attachment.id)}{#if attachment.message == message.id}<li part="attachment" class="attachment" layout="wrap"><og-icon part="icon" class="icon" icon={attachment.icon}></og-icon><div part="name">{attachment.name}</div><div part="figure" class="amount">{attachment.amount}</div></li>{/if}{/each}
        </ul><span part="caption" class="muted state" class:state-claimed={(message.expired ? 'expired' : message.claimed ? 'claimed' : 'available') == 'claimed'} class:state-expired={(message.expired ? 'expired' : message.claimed ? 'claimed' : 'available') == 'expired'}>{message.expired ? t("meta.inbox.expired") : message.claimed ? t("meta.claimed") : t("meta.available")}</span></section>{/if}
        <footer part="actions" class="actions" layout="wrap">
          {#if message.hasAttachments}<button id="Claim" tone="reward" class="go" part="claim" variant="primary" disabled={busy || !message.claimable || message.claimed || message.expired} onclick={claim}>{t("meta.inbox.claim")}</button>{/if}
          {#if message.canDelete}<button id="Delete" part="delete" disabled={busy} onclick={delete}>{t("meta.inbox.delete")}</button>{/if}
        </footer>
      </article>{/if}{/each}
    </div>
  </div>{/if}
</section>

<style>
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(var(--layout-tile), 100%), 1fr)); }
#Inbox { block-size: auto; }
h2, h3, h4, p { margin: 0; overflow-wrap: anywhere; }
.muted { overflow-wrap: anywhere; color: var(--quest-ink-quiet); }
/* The count on a dark cut tag. */
.summary { padding: var(--space-xxs) var(--space-sm); color: var(--ink-bright); background: var(--quest-well-fill); box-shadow: var(--quest-well-rim); border-radius: var(--quest-cut-sm) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.layout { grid-template-columns: minmax(0, 1fr) minmax(0, 1.5fr); }
.list { max-block-size: var(--measure-xl); margin: 0; padding: var(--space-xxs); list-style: none; }
/* A message: a card with two corners cut; unread carries a gold rail, the one open a gold wash. */
.row { display: block; inline-size: stretch; padding: var(--space-sm) var(--space-md); text-align: start; background: var(--quest-entry-fill); box-shadow: var(--quest-entry-rim); border-radius: var(--quest-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.row.unread { box-shadow: var(--quest-selected-rim); }
.row:state(selected) { background: var(--alert-open-fill); box-shadow: var(--quest-selected-rim); }
.row.not-unread .subject { color: var(--quest-ink-quiet); }
.row-copy, .detail { align-items: start; }
.row-copy { inline-size: stretch; }
.subject { inline-size: stretch; font-weight: var(--quest-weight); overflow-wrap: anywhere; line-clamp: 1; text-overflow: ellipsis; }
/* Read or unread, as a small word in the tag's colour. */
.badge { font: var(--caption-text); font-weight: var(--quest-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--quest-ink-faint); }
.row.unread .badge { color: var(--quest-gold); }
.icon { inline-size: var(--size-xs); block-size: var(--size-xs); flex: none; color: var(--quest-gold); }
/* The open message in a sunken reader. */
.reader { min-inline-size: 0; max-block-size: var(--measure-xl); overflow-y: auto; padding: var(--space-md) var(--space-lg); background: var(--quest-well-fill); box-shadow: var(--quest-well-rim); border-radius: var(--alert-reader-radius); corner-shape: var(--well-corner-shape); }
.body { white-space: pre-wrap; overflow-wrap: anywhere; color: var(--quest-ink-quiet); }
.attachments { inline-size: stretch; }
h4 { font: var(--caption-text); font-weight: var(--quest-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--quest-ink-quiet); }
.attachment-list { flex-direction: row; flex-wrap: wrap; margin: 0; padding: 0; list-style: none; }
/* An attachment: a dark cut chip, its icon in gold and its amount as a figure. */
.attachment { flex-wrap: nowrap; align-items: center; padding: var(--space-xs) var(--space-md) var(--space-xs) var(--space-sm); background: var(--quest-well-fill); box-shadow: var(--quest-well-ready-rim); border-radius: var(--quest-cut-sm) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.amount { font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--quest-gold); }
.state.state-expired { color: var(--tone-danger); }
.state.state-claimed { color: var(--tone-success); }
.actions button { min-block-size: var(--touch-target); white-space: normal; }
/* Claim: the reward's gold face. */
.wait { padding: var(--space-lg); }
/* A narrow inbox: the list over the reader in one column, neither scrolling sideways. */
@container (width < 600px) { .layout { grid-template-columns: minmax(0, 1fr); } .list { min-block-size: calc(160 * var(--font-step-px)); max-block-size: var(--size-5xl); } .reader { max-block-size: none; overflow: visible; } }
</style>
