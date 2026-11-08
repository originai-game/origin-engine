<meta name="component" content="ActionHints">
<meta name="strings" content="./action-hints.en.ogt">
<meta name="prop:hints" content="list" data-doc="The hints, a keyed collection of { action, hint, hold } as og.ui.actionBar answers them: action the og-input action whose glyph its cap shows on the device used last, hint what it does (Buy), hold true when it fires on a hold (its fill grows while the action or a press holds it).">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the bar; empty is Actions.">
<meta name="action:press" content="Do">
<meta name="action:hold" content="Hold">
<meta name="fixture:default" content='{"label":"","hints":[{"action":"confirm","hint":"Buy","hold":false},{"action":"back","hint":"Back","hold":false},{"action":"sell","hint":"Sell all","hold":true}]}'>

<nav id="ActionHints" role="toolbar" aria-label={label || t("component.action-hints.name")}>
  {#each hints as h (h.action)}
    <div id="Item" class="item">
      {#if h.hold}
        <button id="HoldHint" class="hint hold" data-hold="800" data-action={h.action} onhold={hold} aria-label={h.hint}>
          <kbd class="cap" data-action={h.action}></kbd>
          <span class="words">{h.hint}</span>
        </button>
      {:else}
        <button id="Hint" class="hint" onclick={press} aria-label={h.hint}>
          <kbd class="cap" data-action={h.action}></kbd>
          <span class="words">{h.hint}</span>
        </button>
      {/if}
    </div>
  {/each}
</nav>

<style>
#ActionHints { flex-direction: row; flex-wrap: wrap; justify-content: end; align-items: center; gap: 16px; padding: 4px 12px; border-radius: 8px; background: #0b0f17b8; }
.item { align-items: center; }
.hint { min-height: 36px; padding: 2px 6px; gap: 8px; align-items: center; border-radius: 6px; background: #00000000; color: var(--ink); }
.hint:hover { background: #ffffff1a; }
.hint:active { scale: 0.96; }
.hint::-og-hold { background: #ffffff33; }
.cap { min-width: 26px; height: 26px; }
.words { font: var(--text-sm); white-space: nowrap; }
@media (pointer: coarse) {
  #ActionHints { justify-content: center; border-radius: 0; gap: 8px; }
  .hint { min-height: 44px; padding: 2px 12px; background: var(--control-strong); }
}
</style>
