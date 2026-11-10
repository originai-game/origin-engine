<meta name="screen" content="notifications" data-layer="toast">
<meta name="prop:items" content="list" data-doc="The notifications shown, newest last, a keyed collection of { id, title, body, tone (info, success, warn, danger) }; the game removes one when it is read or times out.">
<meta name="action:dismiss" content="Dismiss">
<meta name="fixture:default" content="{&quot;items&quot;:[{&quot;id&quot;:&quot;1&quot;,&quot;title&quot;:&quot;Achievement&quot;,&quot;body&quot;:&quot;First blood&quot;,&quot;tone&quot;:&quot;success&quot;}]}">

<column id="Notifications" role="log" aria-live="polite">
  {#each $items as n (n.id)}
    <article class="note" class:success={n.tone == "success"} class:warn={n.tone == "warn"} class:danger={n.tone == "danger"}>
      <div class="text"><p class="title">{n.title}</p><small class="body">{n.body}</small></div>
      <button id="Close" variant="quiet" onclick={dismiss} aria-label={n.title}><og-icon name="close"/></button>
    </article>
  {/each}
</column>

<style>
#Notifications { width: stretch; height: stretch; align-items: end; padding: 16px; gap: 8px; }
.note { display: flex; align-items: center; gap: 8px; width: 320px; padding: 10px 12px; background: var(--panel-raised); border-radius: var(--radius-panel); border-left: 4px solid var(--info); }
.note.success { border-left-color: var(--success); }
.note.warn { border-left-color: var(--warn); }
.note.danger { border-left-color: var(--danger); }
.note { animation: arrive 240ms ease-out both; }
@keyframes arrive { from { translate: 24px 0; opacity: 0; } to { translate: 0 0; opacity: 1; } }
.text { width: stretch; gap: 2px; }
.title { margin: 0; }
.body { color: var(--ink-muted); }
</style>
