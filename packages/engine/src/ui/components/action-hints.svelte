<meta name="component" content="ActionHints">
<meta name="strings" content="./action-hints.en.ogt">
<meta name="parts" content="ActionHints.frame ActionHints.row ActionHints.cap ActionHints.label">
<meta name="prop:hints" content="list" data-doc="The hints, a keyed collection of { action, hint, hold } as og.ui.actionBar answers them: action the og-input action whose glyph its cap shows on the device used last, hint what it does (Buy), hold true when it fires on a hold (its fill grows while the action or a press holds it).">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the bar; empty is Actions.">
<meta name="action:press" content="Do">
<meta name="action:hold" content="Hold">
<meta name="fixture:default" content='{"label":"","hints":[{"action":"confirm","hint":"Buy","hold":false},{"action":"back","hint":"Back","hold":false},{"action":"details","hint":"Sell all","hold":true}]}'>

<nav id="ActionHints" part="ActionHints.frame" class="hud" role="toolbar" aria-label={label || t("component.action-hints.name")}>
  {#each hints as h (h.action)}
    <div id="Item" class="item">
      {#if h.hold}
        <button id="HoldHint" part="ActionHints.row" class="hint hold" data-hold="800" data-action={h.action} onhold={hold} aria-label={h.hint}>
          <kbd part="ActionHints.cap" class="cap" data-action={h.action}></kbd>
          <span part="ActionHints.label" class="words">{h.hint}</span>
        </button>
      {:else}
        <button id="Hint" part="ActionHints.row" class="hint" onclick={press} aria-label={h.hint}>
          <kbd part="ActionHints.cap" class="cap" data-action={h.action}></kbd>
          <span part="ActionHints.label" class="words">{h.hint}</span>
        </button>
      {/if}
    </div>
  {/each}
</nav>

<style>
#ActionHints { flex-direction: row; flex-wrap: wrap; justify-content: end; align-items: center; gap: var(--space-lg); }
.item { align-items: center; }
.hint { min-height: var(--control-lg); padding: var(--space-xxs) var(--space-sm); gap: var(--space-sm); align-items: center; }
.hint:active { scale: var(--state-press-scale); }
.hint::-og-hold { background: var(--selection); }
.cap { min-width: var(--size-sm); height: var(--size-sm); }
.words { white-space: nowrap; }
@media (pointer: coarse) {
  #ActionHints { justify-content: center; border-radius: 0; gap: var(--space-sm); }
  .hint { min-height: var(--touch-target); padding: var(--space-xxs) var(--space-md); background: var(--control-strong); }
}
</style>
