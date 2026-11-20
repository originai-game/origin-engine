<meta name="component" content="ActionHints">
<meta name="strings" content="./action-hints.en.ogt">
<meta name="prop:hints" content="list" data-doc="The hints, a keyed collection of { action, hint, hold } as og.ui.actionBar answers them: action the og-input action whose glyph its cap shows on the device used last, hint what it does (Buy), hold true when it fires on a hold (its fill grows while the action or a press holds it).">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the bar; empty is Actions.">
<meta name="action:press" content="Do">
<meta name="action:hold" content="Hold">
<meta name="fixture:default" content='{"label":"","hints":[{"action":"confirm","hint":"Buy","hold":false},{"action":"back","hint":"Back","hold":false},{"action":"details","hint":"Sell all","hold":true}]}'>

<nav id="ActionHints" part="frame" variant="hud" layout="wrap" gap="wide" role="toolbar" aria-label={label || t("component.action-hints.name")}>
  {#each hints as h (h.action)}
    <div class="item" layout="stack" gap="none">
      {#if h.hold}
        <button part="row" class="hint hold" data-hold="800" data-action={h.action} onhold={hold} aria-label={h.hint}>
          <div class="face" layout="row">
          <kbd part="cap" class="cap" data-action={h.action}></kbd>
          <span part="label" class="words">{h.hint}</span>
          </div>
        </button>
      {:else}
        <button part="row" class="hint" onclick={press} aria-label={h.hint}>
          <div class="face" layout="row">
          <kbd part="cap" class="cap" data-action={h.action}></kbd>
          <span part="label" class="words">{h.hint}</span>
          </div>
        </button>
      {/if}
    </div>
  {/each}
</nav>

<style>
#ActionHints { justify-content: end; }
.item { align-items: center; max-inline-size: 100%; }
.hint { max-inline-size: 100%; min-height: var(--control-lg); padding: var(--space-xxs) var(--space-sm); }
.cap { min-width: var(--size-sm); height: var(--size-sm); flex: none; }
.words { text-align: start; overflow-wrap: anywhere; }
#ActionHints:touch { justify-content: center; --layout-gap-wide: var(--layout-gap); }
.hint:touch { min-height: var(--touch-target); padding: var(--space-xxs) var(--space-md); }
</style>
