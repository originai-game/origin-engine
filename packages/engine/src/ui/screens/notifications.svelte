<meta name="screen" content="notifications" data-layer="toast">
<meta name="parts" content="notifications.frame notifications.body">
<meta name="prop:items" content="list" data-doc="The notifications shown, newest last, a keyed collection of { id, title, body, tone (info, success, warn, danger) }; the game removes one when it is read or times out.">
<meta name="action:dismiss" content="Dismiss">
<meta name="fixture:default" content="{&quot;items&quot;:[{&quot;id&quot;:&quot;1&quot;,&quot;title&quot;:&quot;Achievement&quot;,&quot;body&quot;:&quot;First blood&quot;,&quot;tone&quot;:&quot;success&quot;}]}">

<column id="Notifications" role="log" aria-live="polite">
  {#each $items as n (n.id)}
    <article class="note" part="notifications.frame" class:success={n.tone == "success"} class:warn={n.tone == "warn"} class:danger={n.tone == "danger"}>
      <div class="text"><p class="title">{n.title}</p><small class="body" part="notifications.body">{n.body}</small></div>
      <button id="Close" variant="quiet" onclick={dismiss} aria-label={n.title}><og-icon icon="close"/></button>
    </article>
  {/each}
</column>

<style>
#Notifications { width: stretch; height: stretch; align-items: end; padding: var(--space-lg); gap: var(--space-sm); }
.note { display: flex; align-items: center; gap: var(--space-sm); width: var(--measure-md); border-left: var(--stroke-thick) solid var(--info); }
.note.success { border-left-color: var(--success); }
.note.warn { border-left-color: var(--warn); }
.note.danger { border-left-color: var(--danger); }
.note { animation: arrive var(--motion-enter) both; }
@keyframes arrive { from { translate: var(--space-xl) 0; opacity: 0; } to { translate: 0 0; opacity: 1; } }
.text { width: stretch; gap: var(--space-xxs); }
.title { margin: 0; }
.body { color: var(--ink-muted); }
</style>
