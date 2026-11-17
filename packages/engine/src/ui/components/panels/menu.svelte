<meta name="component" content="Menu">
<meta name="strings" content="./menu.en.ogt">
<meta name="parts" content="Menu.frame Menu.row Menu.heading Menu.separator Menu.icon Menu.check Menu.dot Menu.shortcut Menu.chevron Menu.caption Menu.glyph">
<meta name="prop:entries" content="list" data-doc="Its entries, submenus' too, a keyed collection of { id, label, kind, icon, shortcut, checked, danger, disabled, parent, more }: kind item, check, radio, separator or header; icon an icon name before the label (empty: none); shortcut shown at the end (Ctrl+S); checked a check or radio entry is on; danger a destructive command in the danger ink; disabled shown but inert; parent the id of the entry whose submenu it is in (empty: the top level); more it opens a submenu (a chevron at its end).">
<meta name="prop:submenu" content="string" data-doc="The id of the entry whose submenu shows, its rows in place of the menu's under a back row (set it to an entry with more when choose names that entry; back clears it); empty: the top level.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls it; empty is Menu.">
<meta name="action:choose" content="Choose">
<meta name="action:back" content="Back">
<meta name="fixture:default" content='{"submenu":"","label":"","entries":[{"id":"h","label":"Item","kind":"header","icon":"","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"","more":false},{"id":"copy","label":"Copy","kind":"item","icon":"plus","shortcut":"Ctrl+C","checked":false,"danger":false,"disabled":false,"parent":"","more":false},{"id":"compare","label":"Compare","kind":"check","icon":"","shortcut":"C","checked":true,"danger":false,"disabled":false,"parent":"","more":false},{"id":"sort","label":"Sort by","kind":"item","icon":"flag","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"","more":true},{"id":"sell","label":"Sell","kind":"item","icon":"coin","shortcut":"","checked":false,"danger":false,"disabled":true,"parent":"","more":false},{"id":"s1","label":"","kind":"separator","icon":"","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"","more":false},{"id":"destroy","label":"Destroy","kind":"item","icon":"bomb","shortcut":"Del","checked":false,"danger":true,"disabled":false,"parent":"","more":false},{"id":"name","label":"Name","kind":"radio","icon":"","shortcut":"","checked":true,"danger":false,"disabled":false,"parent":"sort","more":false},{"id":"value","label":"Value","kind":"radio","icon":"","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"sort","more":false}]}'>

<menu id="Menu" part="Menu.frame" data-layout="stack" data-gap="none" class="bare" role="menu" aria-label={label || t("component.menu.name")}>
  {#if submenu != ""}
    <button id="Back" part="Menu.row" class="entry back" role="menuitem" onclick={back}><div class="face" data-layout="row">
      <svg class="chevron" part="Menu.chevron" viewBox="0 0 10 10"><polyline part="Menu.glyph" points="6.5,1.5 3,5 6.5,8.5"></polyline></svg>
      <span class="name" data-fill>{t("component.menu.back")}</span>
    </div></button>
    <hr class="separator" part="Menu.separator">
  {/if}
  {#each entries as e (e.id)}
    {#if e.parent == submenu}<div class="row" data-layout="stack" data-gap="none">
      {#if e.kind == "separator"}
        <hr class="separator" part="Menu.separator">
      {:else if e.kind == "header"}
        <span class="header" part="Menu.heading">{e.label}</span>
      {:else}
        <button part="Menu.row" class="entry" class:danger={e.danger} data-tone={e.danger && !e.disabled ? 'danger' : ''} role={e.kind == "check" ? "menuitemcheckbox" : e.kind == "radio" ? "menuitemradio" : "menuitem"} aria-checked={e.kind == "check" || e.kind == "radio" ? e.checked : null} aria-label={e.label} disabled={e.disabled} onclick={choose}><div class="face" data-layout="row">
          <div class="slot" data-layout="cover" data-gap="none">
            {#if e.icon != "" && e.kind != "check" && e.kind != "radio"}<og-icon class="glyph" part="Menu.icon" icon={e.icon}></og-icon>{/if}
            {#if e.kind == "check" && e.checked}<svg class="check" part="Menu.check" viewBox="0 0 12 12"><polyline part="Menu.glyph" points="2,6.5 5,9.5 10,3"></polyline></svg>{/if}
            {#if e.kind == "radio" && e.checked}<span class="dot" part="Menu.dot"></span>{/if}
          </div>
          <span class="name" data-fill>{e.label}</span>
          {#if e.shortcut}<span class="shortcut" part="Menu.shortcut Menu.caption">{e.shortcut}</span>{/if}
          {#if e.more}<svg class="chevron" part="Menu.chevron" viewBox="0 0 10 10"><polyline part="Menu.glyph" points="3.5,1.5 7,5 3.5,8.5"></polyline></svg>{/if}
        </div></button>
      {/if}
    </div>{/if}
  {/each}
</menu>

<style>
:host { min-inline-size: var(--measure-xs); max-inline-size: min(100%, var(--measure-md)); margin: 0; padding: var(--space-xs); list-style: none; }
[part="Menu.row"] { width: stretch; height: var(--row); padding: 0 var(--space-md); min-width: 0; }
.row { width: stretch; }
.face { width: stretch; height: stretch; }
.slot { width: var(--size-xs); height: var(--size-xs); }
[part="Menu.icon"] { width: var(--size-xs); height: var(--size-xs); }
[part="Menu.check"] { width: 75%; height: 75%; }
[part="Menu.dot"] { --dot-fill: var(--primary); width: 50%; height: 50%; }
.name { text-align: start; white-space: nowrap; text-overflow: ellipsis; }
[part~="Menu.shortcut"] { white-space: nowrap; }
[part="Menu.chevron"] { width: var(--font-step-2xs); height: var(--font-step-2xs); }
[part="Menu.chevron"]:dir(rtl) { scale: -1 1; }
[part="Menu.heading"] { padding: var(--space-sm) var(--space-md) var(--space-xs); }
[part="Menu.separator"] { width: stretch; margin: var(--space-xs) 0; }
</style>
