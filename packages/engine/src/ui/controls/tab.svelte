<meta name="control" content="input[type=radio][role=tab]">
<meta name="parts" content="label">
<meta name="prop:text" content="string" data-doc="Its label (its <label>'s text).">
<meta name="prop:icon" content="string" data-doc="An icon before the label (data-icon).">
<meta name="prop:badge" content="string" data-doc="A count or word after the label (data-badge).">

<div id="Face" class="face">
  {#if icon}<og-icon id="Icon" class="icon" icon={icon} aria-label=""></og-icon>{/if}
  {#if text}<span id="Label" part="label">{text}</span>{/if}
  {#if badge}<badge id="Badge" class="badge" tone="accent">{badge}</badge>{/if}
</div>

<style>
:host { display: block; cursor: pointer; }
:host(:disabled) { cursor: default; }
.face {
  display: flex; align-items: center; justify-content: center; gap: var(--space-xs);
  block-size: 100%; padding-inline: var(--space-md);
  white-space: nowrap;
}
.icon { inline-size: var(--size-xs); block-size: var(--size-xs); flex: none; }
</style>
