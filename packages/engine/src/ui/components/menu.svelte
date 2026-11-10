<meta name="component" content="Menu">
<meta name="strings" content="./menu.en.ogt">
<meta name="prop:entries" content="list" data-doc="Its entries, submenus' too, a keyed collection of { id, label, kind, icon, shortcut, checked, danger, disabled, parent, more }: kind item, check, radio, separator or header; icon an icon name before the label (empty: none); shortcut shown at the end (Ctrl+S); checked a check or radio entry is on; danger a destructive command in the danger ink; disabled shown but inert; parent the id of the entry whose submenu it is in (empty: the top level); more it opens a submenu (a chevron at its end).">
<meta name="prop:submenu" content="string" data-doc="The id of the entry whose submenu shows, its rows in place of the menu's under a back row (set it to an entry with more when choose names that entry; back clears it); empty: the top level.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls it; empty is Menu.">
<meta name="action:choose" content="Choose">
<meta name="action:back" content="Back">
<meta name="fixture:default" content='{"submenu":"","label":"","entries":[{"id":"h","label":"Item","kind":"header","icon":"","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"","more":false},{"id":"copy","label":"Copy","kind":"item","icon":"plus","shortcut":"Ctrl+C","checked":false,"danger":false,"disabled":false,"parent":"","more":false},{"id":"compare","label":"Compare","kind":"check","icon":"","shortcut":"C","checked":true,"danger":false,"disabled":false,"parent":"","more":false},{"id":"sort","label":"Sort by","kind":"item","icon":"flag","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"","more":true},{"id":"sell","label":"Sell","kind":"item","icon":"coin","shortcut":"","checked":false,"danger":false,"disabled":true,"parent":"","more":false},{"id":"s1","label":"","kind":"separator","icon":"","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"","more":false},{"id":"destroy","label":"Destroy","kind":"item","icon":"bomb","shortcut":"Del","checked":false,"danger":true,"disabled":false,"parent":"","more":false},{"id":"name","label":"Name","kind":"radio","icon":"","shortcut":"","checked":true,"danger":false,"disabled":false,"parent":"sort","more":false},{"id":"value","label":"Value","kind":"radio","icon":"","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"sort","more":false}]}'>

<menu id="Menu" role="menu" aria-label={label || t("component.menu.name")}>
  {#if submenu != ""}
    <button id="Back" class="entry back" role="menuitem" onclick={back}><div class="face">
      <svg class="chevron" viewBox="0 0 10 10"><polyline points="6.5,1.5 3,5 6.5,8.5"></polyline></svg>
      <span class="name">{t("component.menu.back")}</span>
    </div></button>
    <hr class="separator">
  {/if}
  {#each entries as e (e.id)}
    <div class="row" hidden={e.parent != submenu}>
      {#if e.kind == "separator"}
        <hr class="separator">
      {:else if e.kind == "header"}
        <span class="header">{e.label}</span>
      {:else}
        <button class="entry" class:danger={e.danger} role={e.kind == "check" ? "menuitemcheckbox" : e.kind == "radio" ? "menuitemradio" : "menuitem"} aria-checked={e.kind == "check" || e.kind == "radio" ? e.checked : null} aria-label={e.label} disabled={e.disabled} onclick={choose}><div class="face">
          <div class="slot">
            <og-icon class="glyph" icon={e.icon} hidden={e.icon == "" || e.kind == "check" || e.kind == "radio"}></og-icon>
            <svg class="check" viewBox="0 0 12 12" hidden={e.kind != "check" || !e.checked}><polyline points="2,6.5 5,9.5 10,3"></polyline></svg>
            <span class="dot" hidden={e.kind != "radio" || !e.checked}></span>
          </div>
          <span class="name">{e.label}</span>
          <span class="shortcut" hidden={e.shortcut == ""}>{e.shortcut}</span>
          <svg class="chevron" viewBox="0 0 10 10" hidden={!e.more}><polyline points="3.5,1.5 7,5 3.5,8.5"></polyline></svg>
        </div></button>
      {/if}
    </div>
  {/each}
</menu>

<style>
#Menu { display: flex; flex-direction: column; min-width: 200px; max-width: 360px; margin: 0; padding: 4px; gap: 0; list-style: none; }
.entry { width: stretch; height: 40px; padding: 0 10px; border-radius: 8px; background: #00000000; color: var(--ink); }
.row { display: flex; flex-direction: column; width: stretch; }
.face { display: flex; align-items: center; gap: 8px; width: stretch; height: stretch; }
.entry:hover, .entry:focus-visible { background: var(--control); }
.entry:disabled { color: var(--ink-disabled); }
.danger { color: var(--danger); }
.slot { display: flex; width: 18px; height: 18px; align-items: center; justify-content: center; }
.glyph { width: 18px; height: 18px; color: var(--ink-muted); }
.danger .glyph { color: var(--danger); }
.check { width: 12px; height: 12px; fill: none; stroke: var(--primary); stroke-width: 2px; }
.dot { width: 8px; height: 8px; border-radius: 4px; background: var(--primary); }
.name { width: stretch; font: var(--text-base); white-space: nowrap; text-overflow: ellipsis; }
.shortcut { font: var(--text-sm); color: var(--ink-muted); white-space: nowrap; }
.chevron { width: 10px; height: 10px; fill: none; stroke: var(--ink-muted); stroke-width: 1.5px; }
.back .name { color: var(--ink-muted); }
.header { padding: 8px 10px 4px; font: var(--text-caption); text-transform: uppercase; color: var(--ink-muted); }
.separator { width: stretch; margin: 4px 0; }
</style>
