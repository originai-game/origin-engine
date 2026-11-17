<meta name="component" content="ItemSlot">
<meta name="strings" content="./item-slot.en.ogt">
<meta name="parts" content="ItemSlot.frame ItemSlot.cell ItemSlot.icon ItemSlot.plus ItemSlot.loading ItemSlot.count ItemSlot.cooldown ItemSlot.remaining ItemSlot.figure ItemSlot.key ItemSlot.state ItemSlot.badge ItemSlot.fill ItemSlot.caption">
<meta name="prop:name" content="string" data-default="" data-doc="The item name, also its accessible name and tooltip. Empty slots use the localized empty-slot name.">
<meta name="prop:icon" content="string" data-default="" data-doc="An atlas icon. Empty is no item art.">
<meta name="prop:count" content="number" data-default="1" data-doc="The quantity; a badge appears above 1.">
<meta name="prop:rarity" content="string" data-default="common" data-doc="The item's rarity: common, uncommon, rare, epic, legendary or mythic; a pack selects data-rarity.">
<meta name="prop:cooldown" content="number" data-default="0" data-doc="Seconds remaining, supplied by the game. A cooling item can still be selected for inspection.">
<meta name="prop:cooldownMax" content="number" data-default="1" data-doc="The cooldown's full duration; the overlay shows the remaining fraction.">
<meta name="prop:keyLabel" content="string" data-default="" data-doc="A shortcut label; empty hides it.">
<meta name="prop:selected" content="bool" data-default="false" data-doc="The game-selected slot, exposed as aria-pressed.">
<meta name="prop:disabled" content="bool" data-default="false" data-doc="Unavailable; selection is disabled.">
<meta name="prop:locked" content="bool" data-default="false" data-doc="Locked; shown in words and selection is disabled.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Waiting for item data; selection is disabled.">
<meta name="prop:empty" content="bool" data-default="false" data-doc="An empty slot. It can be selected so the game can offer an equip action.">
<meta name="prop:showName" content="bool" data-default="false" data-doc="Show the wrapping item name below the slot.">
<meta name="fixture:default" content='{"name":"Healing draught","icon":"potion","count":12,"rarity":"rare","cooldown":0,"cooldownMax":8,"keyLabel":"Q","selected":false,"disabled":false,"locked":false,"loading":false,"empty":false,"showName":true}'>
<meta name="fixture:selected" content='{"name":"Storm crystal","icon":"gem","count":3,"rarity":"epic","cooldown":3,"cooldownMax":8,"keyLabel":"2","selected":true,"disabled":false,"locked":false,"loading":false,"empty":false,"showName":true}'>
<meta name="fixture:locked" content='{"name":"Sealed relic","icon":"key","rarity":"legendary","locked":true,"showName":true}'>
<meta name="fixture:disabled" content='{"name":"Spent potion","icon":"potion","disabled":true,"showName":true}'>
<meta name="fixture:loading" content='{"loading":true,"showName":true}'>
<meta name="fixture:empty" content='{"empty":true,"showName":true}'>

<!-- Reports choose. In a keyed collection the intent keeps its enclosing
     row key; otherwise the game's stable component instance identifies it. -->
<div id="ItemSlot" part="ItemSlot.frame" class="bare" data-layout="stack" data-gap="tight">
  <button id="Slot" part="ItemSlot.cell" class:cooling={cooldown > 0 && !empty && !loading} data-rarity={rarity} data-empty={empty} aria-label={t("base.item-slot.label", { name: name || t("base.item-slot.empty"), state: loading ? t("base.item-slot.loading") : locked ? t("base.item-slot.locked") : disabled ? t("base.item-slot.disabled") : cooldown > 0 ? t("base.item-slot.cooldown", { time: formatCountdown(cooldown) }) : empty ? t("base.item-slot.empty") : t("base.item-slot.count", { count }) })} aria-pressed={selected} aria-busy={loading} title={name} disabled={disabled || locked || loading} onclick={choose}>
    {#if icon && !empty && !loading}<og-icon part="ItemSlot.icon" icon={icon}></og-icon>{/if}
    {#if (empty || !icon) && !loading}<og-icon part="ItemSlot.plus ItemSlot.caption" class="plus" icon="plus"></og-icon>{/if}
    {#if loading}<div part="ItemSlot.loading" data-layout="cover"><Spinner name={t("base.item-slot.loading")} /></div>{/if}
    {#if cooldown > 0 && !empty && !loading}<div part="ItemSlot.cooldown ItemSlot.fill" class="cooldown" style:--remaining={cooldownMax > 0 ? cooldown / cooldownMax : 0}></div><div part="ItemSlot.remaining ItemSlot.figure" class="remaining" data-anchor="center">{formatCountdown(cooldown)}</div>{/if}
    {#if count > 1 && !empty && !loading}<div part="ItemSlot.count ItemSlot.badge" class="count">{formatNumber(count)}</div>{/if}
    {#if keyLabel}<div part="ItemSlot.key ItemSlot.badge" class="key">{keyLabel}</div>{/if}
    {#if (locked || disabled) && !loading}<div part="ItemSlot.state ItemSlot.badge" class="state">{locked ? t("base.item-slot.locked") : t("base.item-slot.disabled")}</div>{/if}
  </button>
  {#if showName}<span part="ItemSlot.caption" class="name">{name || (loading ? t("base.item-slot.loading") : t("base.item-slot.empty"))}</span>{/if}
</div>

<style>
:host { inline-size: var(--item-slot-size, var(--size-xxl)); }
[part="ItemSlot.cell"] { position: relative; inline-size: 100%; block-size: var(--item-slot-size, var(--size-xxl)); min-block-size: var(--size-touch); padding: var(--space-sm); overflow: hidden; }
[part="ItemSlot.icon"] { inline-size: 64%; block-size: 64%; }
.plus { inline-size: 36%; block-size: 36%; }
.cooling [part="ItemSlot.icon"] { opacity: var(--cell-cooling-opacity); }
.cooldown { position: absolute; inset-inline: 0; bottom: 0; block-size: clamp(0%, calc(var(--remaining) * 100%), 100%); --fill-color: var(--scrim); --fill-gloss: none; pointer-events: none; }
.count, .key { position: absolute; padding-inline: var(--space-xs); }
.count { inset-inline-end: var(--space-xxs); bottom: var(--space-xxs); }
.key { inset-inline-start: var(--space-xxs); top: var(--space-xxs); }
.remaining { --figure-size: var(--type-lg-size); --figure-weight: var(--font-weight-bold); }
.state { position: absolute; inset-inline: var(--space-xxs); bottom: var(--space-xxs); padding: var(--space-xxs); text-align: center; }
[part="ItemSlot.loading"] { position: absolute; inset: 0; --spinner-size: calc(var(--item-slot-size, var(--size-xxl)) * 0.4); }
.name { text-align: center; overflow-wrap: anywhere; }
</style>
