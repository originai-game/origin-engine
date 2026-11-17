<meta name="component" content="ListItem">
<meta name="strings" content="./states.en.ogt">
<meta name="parts" content="ListItem.row ListItem.well ListItem.picture ListItem.icon ListItem.body ListItem.title ListItem.subtitle ListItem.trailing ListItem.figure ListItem.badge ListItem.mark ListItem.headline ListItem.caption">
<meta name="prop:title" content="string" data-default="" data-doc="The row's primary text.">
<meta name="prop:subtitle" content="string" data-default="" data-doc="Optional secondary text, wrapping below the title.">
<meta name="prop:icon" content="string" data-default="" data-doc="Optional leading icon, used when image is zero.">
<meta name="prop:image" content="number" data-default="0" data-doc="Optional loaded portrait or item texture handle.">
<meta name="prop:value" content="string" data-default="" data-doc="The game's formatted trailing value.">
<meta name="prop:badge" content="string" data-default="" data-doc="Optional trailing badge, such as New or 3.">
<meta name="prop:selected" content="bool" data-default="false" data-doc="Chosen by the game; marked with a check and aria-pressed.">
<meta name="prop:disabled" content="bool" data-default="false" data-doc="Unavailable; cannot choose.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Refreshing this row; prevents choosing and shows a spinner without hiding its content.">
<meta name="action:choose" content="Choose this row (the enclosing collection's stable key)">
<meta name="fixture:default" content='{"title":"Ember Blade","subtitle":"Legendary sword · Equipped by Mira","icon":"sword","value":"1,250","badge":"New","selected":true}'>
<meta name="fixture:disabled" content='{"title":"Northern expedition","subtitle":"Reach level 12 to unlock this destination.","icon":"lock","disabled":true}'>
<meta name="fixture:loading" content='{"title":"Shared build","subtitle":"Retrieving equipment…","icon":"bag","loading":true}'>
<meta name="fixture:overflow" content='{"title":"The Ember Blade of the Last Watcher of the Ninefold Gate","subtitle":"A two-handed legendary sword with an unusually long description that must remain readable on a phone.","icon":"sword","value":"1,250,000 gold","badge":"Recommended","selected":true}'>

<div data-layout="shell">
<button id="ListItem" part="ListItem.row" class="item" class:selected={selected} aria-label={title} aria-description={subtitle} aria-pressed={selected} aria-busy={loading} disabled={disabled || loading} onclick={choose}>
  <div class="face" data-layout="cluster">
  {#if image || icon}<div part="ListItem.well" class="well" data-layout="cover">
    {#if image}<img part="ListItem.picture" class="picture" src={image} alt="">
    {:else}<og-icon part="ListItem.icon" class="icon" icon={icon}></og-icon>{/if}
  </div>{/if}
  <div part="ListItem.body" class="body" data-layout="stack" data-gap="hair" data-fill><span part="ListItem.title ListItem.headline" class="title">{title}</span>{#if subtitle}<span part="ListItem.subtitle ListItem.caption" class="subtitle">{subtitle}</span>{/if}</div>
  {#if value || badge || selected || loading}<div part="ListItem.trailing" class="trailing" data-layout="cluster" data-gap="tight" class:hasText={value != '' || badge != ''}>
    {#if value}<span part="ListItem.figure" class="value">{value}</span>{/if}
    {#if badge}<span part="ListItem.badge" class="badge">{badge}</span>{/if}
    {#if loading}<Spinner size={16} name={t("base.loading")} />{:else if selected}<og-icon part="ListItem.mark" class="mark" icon="check"></og-icon>{/if}
  </div>{/if}
  </div>
</button>
</div>

<style>
:host { block-size: auto; }
.item { inline-size: stretch; min-block-size: var(--touch-target); padding: var(--space-sm) var(--space-md); text-align: start; }
.face { inline-size: stretch; }
.well { flex: none; inline-size: var(--size-touch); block-size: var(--size-touch); }
.picture { inline-size: 100%; block-size: 100%; object-fit: cover; }
.icon { inline-size: var(--size-md); block-size: var(--size-md); }
.title, .subtitle, .value { overflow-wrap: anywhere; }
.trailing { justify-content: end; max-inline-size: 100%; }
.mark { inline-size: var(--size-xs); block-size: var(--size-xs); flex: none; }
@container (width < 480px) { .trailing.hasText { inline-size: 100%; } }
</style>
