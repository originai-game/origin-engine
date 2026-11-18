<meta name="component" content="StrategyAlerts">
<meta name="strings" content="./strategy.en.ogt">
<meta name="parts" content="StrategyAlerts.frame StrategyAlerts.alert StrategyAlerts.mark StrategyAlerts.text StrategyAlerts.key">
<meta name="prop:alerts" content="list" data-doc="The alerts and notices, newest first, keyed rows of { id, kind, text, key, faded }: kind danger (under attack), done (research complete) or info, on data-kind, each with its own mark and edge; text the game's words; key the input that jumps to it as the game names it (Space), on a key cap, empty none, hidden under a touch screen; faded an older notice, set back (data-faded).">
<meta name="fixture:default" content='{"alerts":[{"id":"a1","kind":"danger","text":"The east tower is under attack","key":"Space","faded":false},{"id":"a2","kind":"done","text":"Research complete: Steel arrowheads","key":"","faded":true}]}'>
<meta name="fixture:info" content='{"alerts":[{"id":"a1","kind":"info","text":"Your ally needs help at the ford","key":"F1","faded":false},{"id":"a2","kind":"done","text":"Barracks complete","key":"","faded":false},{"id":"a3","kind":"danger","text":"Villager killed","key":"","faded":true}]}'>
<meta name="fixture:empty" content='{"alerts":[]}'>
<meta name="fixture:overflow" content='{"alerts":[{"id":"a1","kind":"danger","text":"The Grand Cathedral of the Seventh Concord on the eastern isle is under attack by siege engines","key":"Left Shoulder + Space","faded":false},{"id":"a2","kind":"done","text":"Research complete: Masterwork Steel Arrowheads of the Royal Fletchers Guild","key":"","faded":true},{"id":"a3","kind":"info","text":"Market prices changed","key":"","faded":true},{"id":"a4","kind":"info","text":"A trade caravan arrived","key":"","faded":true}]}'>

<!--
  Alerts and notices at the top left under the resources: one feed row
  each, newest first, lit from the edge it hangs on, a mark and an edge in
  danger, done or info, the game's words and the key that jumps to it; an
  older notice set back. The game owns which alerts stand and for how
  long; read only.
-->
<ul id="StrategyAlerts" part="StrategyAlerts.frame" class="hud bare" aria-label={t("strategy.alerts.name")} role="log">
  {#each alerts as a (a.id)}
    <li part="StrategyAlerts.alert" class="alert" data-kind={a.kind} data-faded={a.faded} aria-label={a.kind == 'danger' ? t("strategy.alerts.danger", { text: a.text }) : (a.kind == 'done' ? t("strategy.alerts.done", { text: a.text }) : a.text)}>
      <span class="edge"></span>
      <span part="StrategyAlerts.mark" class="mark"></span>
      <span part="StrategyAlerts.text" class="text">{a.text}</span>
      {#if a.key != ''}<span part="StrategyAlerts.key" class="key">{a.key}</span>{/if}
    </li>
  {/each}
</ul>

<style>
:host { display: flex; flex-direction: column; align-items: flex-start; row-gap: calc(5 * var(--font-step-px)); margin: 0; padding: 0; list-style: none; inline-size: fit-content; max-inline-size: stretch; min-inline-size: 0; color: var(--hud-ink); }
/* A feed row lit from its start edge, the edge in the alert's colour. */
.alert { display: flex; align-items: center; column-gap: calc(8 * var(--font-step-px)); max-inline-size: 100%; min-inline-size: 0; padding: calc(4 * var(--font-step-px)) calc(12 * var(--font-step-px)) calc(4 * var(--font-step-px)) 0; background: var(--hud-feed-fill-start); }
.edge { flex: none; align-self: stretch; inline-size: var(--hud-feed-edge-width); margin-inline-end: calc(4 * var(--font-step-px)); background: var(--hud-feed-edge); }
[data-kind=danger] > .edge { background: var(--hud-harm); }
[data-kind=done] > .edge { background: var(--hud-boon); }
[data-kind=info] > .edge { background: var(--hud-accent); }
/* The mark: a small diamond in the alert's colour. */
.mark { flex: none; inline-size: calc(8 * var(--font-step-px)); block-size: calc(8 * var(--font-step-px)); transform: rotate(45deg); background: var(--hud-feed-edge); box-shadow: var(--hud-marker-rim); }
[data-kind=danger] .mark { background: var(--hud-harm); box-shadow: var(--hud-foe-glow); }
[data-kind=done] .mark { background: var(--hud-boon); }
[data-kind=info] .mark { background: var(--hud-accent); }
.text { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-size-feed); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; text-shadow: var(--hud-shadow); }
[data-faded=true] .text { color: var(--hud-ink-dim); }
[data-faded=true] .mark { box-shadow: var(--hud-marker-rim); }
.key { flex: 0 1 auto; min-inline-size: calc(22 * var(--font-step-px)); max-inline-size: 40%; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); text-align: center; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
@media (pointer: coarse) { .key { display: none; } }
@media (width < 600px) {
  :host { inline-size: stretch; row-gap: calc(3 * var(--font-step-px)); }
  .alert { column-gap: calc(6 * var(--font-step-px)); padding: calc(3 * var(--font-step-px)) calc(8 * var(--font-step-px)) calc(3 * var(--font-step-px)) 0; }
  .text { font-size: var(--hud-size-xs); }
}
</style>
