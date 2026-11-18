<meta name="screen" content="notifications" data-layer="toast">
<meta name="parts" content="notifications.frame notifications.body notifications.title notifications.mark notifications.toast">
<meta name="prop:items" content="list" data-doc="The game's own notifications, newest last, a keyed collection of { id, title, body, tone (info, success, warn, danger) }; the game removes one when it is read or times out. The toasts og.ui.toast queued ($toasts) follow them, with nothing to pass: each leaves on its own and a press dismisses it.">
<meta name="action:dismiss" content="Dismiss">
<meta name="fixture:default" content="{&quot;items&quot;:[{&quot;id&quot;:&quot;1&quot;,&quot;title&quot;:&quot;Achievement&quot;,&quot;body&quot;:&quot;First blood&quot;,&quot;tone&quot;:&quot;success&quot;}]}">

<!--
  The notification stack in the HUD's top-right corner. A notification is
  a glass plate with two corners cut, a rail down its leading edge and a
  wash from it in its tone, the tone's mark in an octagon socket, its
  title in small capitals of the tone and its body in white, and a quiet
  close. A toast is the compact form: its mark and its words on a plate
  that slides back out when it leaves. Each tone has its own hue (warn an
  amber well apart from gold). Under reduced motion they appear in place.
-->
<div id="Notifications" role="log" aria-live="polite">
  {#each items as n (n.id)}
    <article class="note" part="notifications.frame" data-tone={n.tone}>
      <div class="socket" part="notifications.mark"><div class="socket-face"><og-icon class="glyph" icon={n.tone == 'success' ? 'check' : n.tone == 'warn' ? 'bolt' : n.tone == 'danger' ? 'close' : 'star'}></og-icon></div></div>
      <div class="text"><p class="title" part="notifications.title">{n.title}</p><p class="body" part="notifications.body">{n.body}</p></div>
      <button id="Close" variant="quiet" onclick={dismiss} aria-label={n.title}><og-icon class="close" icon="close"/></button>
    </article>
  {/each}
  {#each $toasts as t (t.id)}
    <article class="note toast" part="notifications.frame notifications.toast" data-tone={t.tone} class:leaving={t.leaving}>
      <button class="said" variant="quiet" onclick={dismissToast}><og-icon class="toast-mark" icon={t.tone == 'success' ? 'check' : t.tone == 'warn' ? 'bolt' : t.tone == 'danger' ? 'close' : 'star'}></og-icon><span class="said-text">{t.text}</span></button>
    </article>
  {/each}
</div>

<style>
#Notifications { display: flex; flex-direction: column; align-items: end; gap: var(--space-sm); block-size: stretch; padding: calc(env(-og-hud-inset-top) + var(--space-lg)) calc(env(-og-hud-inset-right) + var(--space-lg)) calc(env(-og-hud-inset-bottom) + var(--space-lg)) calc(env(-og-hud-inset-left) + var(--space-lg)); }
/* A plate with two corners cut, its tone's rail and wash at its leading edge. */
.note { position: relative; border-radius: 0; display: flex; align-items: center; gap: var(--space-md); inline-size: var(--measure-md); max-inline-size: 100%; padding: var(--space-sm) var(--space-xs) var(--space-sm) calc(var(--alert-rail) + var(--space-md)); background: var(--alert-info-wash), var(--alert-plate-fill); box-shadow: var(--alert-plate-rim); clip-path: polygon(var(--quest-cut) 0, 100% 0, 100% calc(100% - var(--quest-cut)), calc(100% - var(--quest-cut)) 100%, 0 100%, 0 var(--quest-cut)); animation: arrive var(--motion-enter) both; }
.note::before { content: ""; position: absolute; inset-block: 0; inset-inline-start: 0; inline-size: var(--alert-rail); background: var(--alert-info); }
.note[data-tone=success] { background: var(--alert-success-wash), var(--alert-plate-fill); }
.note[data-tone=success]::before { background: var(--alert-success); }
.note[data-tone=warn] { background: var(--alert-warn-wash), var(--alert-plate-fill); }
.note[data-tone=warn]::before { background: var(--alert-warn); }
.note[data-tone=danger] { background: var(--alert-danger-wash), var(--alert-plate-fill); }
.note[data-tone=danger]::before { background: var(--alert-danger); }
/* The tone's mark in an octagon socket with a lit edge. */
.socket { flex: none; display: flex; inline-size: var(--size-lg); block-size: var(--size-lg); padding: var(--stroke-thin); background: var(--quest-socket-rim); clip-path: polygon(22% 0, 78% 0, 100% 22%, 100% 78%, 78% 100%, 22% 100%, 0 78%, 0 22%); }
.socket-face { display: flex; align-items: center; justify-content: center; inline-size: stretch; block-size: stretch; background: var(--quest-socket-fill); clip-path: polygon(22% 0, 78% 0, 100% 22%, 100% 78%, 78% 100%, 22% 100%, 0 78%, 0 22%); color: var(--alert-info); }
.glyph { inline-size: var(--size-xs); block-size: var(--size-xs); }
.note[data-tone=success] .socket-face, .note[data-tone=success] .title, .toast[data-tone=success] .toast-mark { color: var(--alert-success); }
.note[data-tone=warn] .socket-face, .note[data-tone=warn] .title, .toast[data-tone=warn] .toast-mark { color: var(--alert-warn); }
.note[data-tone=danger] .socket-face, .note[data-tone=danger] .title, .toast[data-tone=danger] .toast-mark { color: var(--alert-danger); }
.text { display: flex; flex-direction: column; flex: 1; min-inline-size: 0; gap: var(--space-xxs); }
.title { margin: 0; font: var(--caption-text); font-weight: var(--alert-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--alert-info); overflow-wrap: anywhere; }
.body { margin: 0; font-weight: var(--headline-weight); color: var(--ink-bright); overflow-wrap: anywhere; }
#Close { flex: none; align-self: start; color: var(--ink-faint); }
.close { inline-size: var(--size-xs); block-size: var(--size-xs); }
/* A toast: the compact plate, as wide as its words. */
.toast { inline-size: auto; padding: 0 0 0 var(--alert-rail); }
.said { gap: var(--space-xs); inline-size: stretch; text-align: start; color: var(--ink-bright); }
.said-text { min-inline-size: 0; overflow-wrap: anywhere; }
.toast-mark { flex: none; inline-size: var(--size-xs); block-size: var(--size-xs); color: var(--alert-info); }
.toast.leaving { animation: leave var(--motion-exit) both; }
@keyframes arrive { from { translate: var(--space-xl) 0; opacity: 0; } to { translate: 0 0; opacity: 1; } }
@keyframes leave { to { translate: var(--space-xl) 0; opacity: 0; } }
@keyframes appear { from { opacity: 0; } to { opacity: 1; } }
@keyframes fade { to { opacity: 0; } }
/* Under reduced motion they fade in place. */
@media (prefers-reduced-motion: reduce) {
  .note { animation-name: appear; }
  .toast.leaving { animation-name: fade; }
}
@media (width < 600px) {
  .note { inline-size: stretch; }
  .toast { inline-size: auto; }
}
</style>
