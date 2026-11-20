<meta name="component" content="Menu">
<meta name="strings" content="./menu.en.ogt">
<meta name="prop:entries" content="list" data-doc="Its entries, submenus' too, a keyed collection of { id, label, kind, icon, shortcut, checked, danger, disabled, parent, more }: kind item, check, radio, separator or header; icon an icon name before the label (empty: none); shortcut shown at the end (Ctrl+S); checked a check or radio entry is on; danger a destructive command in the danger ink; disabled shown but inert; parent the id of the entry whose submenu it is in (empty: the top level); more it opens a submenu (a chevron at its end).">
<meta name="prop:submenu" content="string" data-doc="The id of the entry whose submenu shows, its rows in place of the menu's under a back row (set it to an entry with more when choose names that entry; back clears it); empty: the top level.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls it; empty is Menu.">
<meta name="action:choose" content="Choose">
<meta name="action:back" content="Back">
<meta name="fixture:default" content='{"submenu":"","label":"","entries":[{"id":"h","label":"Item","kind":"header","icon":"","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"","more":false},{"id":"copy","label":"Copy","kind":"item","icon":"plus","shortcut":"Ctrl+C","checked":false,"danger":false,"disabled":false,"parent":"","more":false},{"id":"compare","label":"Compare","kind":"check","icon":"","shortcut":"C","checked":true,"danger":false,"disabled":false,"parent":"","more":false},{"id":"sort","label":"Sort by","kind":"item","icon":"flag","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"","more":true},{"id":"sell","label":"Sell","kind":"item","icon":"coin","shortcut":"","checked":false,"danger":false,"disabled":true,"parent":"","more":false},{"id":"s1","label":"","kind":"separator","icon":"","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"","more":false},{"id":"destroy","label":"Destroy","kind":"item","icon":"bomb","shortcut":"Del","checked":false,"danger":true,"disabled":false,"parent":"","more":false},{"id":"name","label":"Name","kind":"radio","icon":"","shortcut":"","checked":true,"danger":false,"disabled":false,"parent":"sort","more":false},{"id":"value","label":"Value","kind":"radio","icon":"","shortcut":"","checked":false,"danger":false,"disabled":false,"parent":"sort","more":false}]}'>

<menu id="Menu" part="frame" layout="stack" gap="none" role="menu" aria-label={label || t("component.menu.name")}>
  {#if submenu != ""}
    <button id="Back" part="row" class="entry back" role="menuitem" onclick={back}><div class="face" layout="row">
      <og-icon class="chevron" part="chevron" icon="chevronLeft"></og-icon>
      <span class="name" fill>{t("component.menu.back")}</span>
    </div></button>
    <hr class="separator" part="separator">
  {/if}
  {#each entries as e (e.id)}
    {#if e.parent == submenu}<div class="row" layout="stack" gap="none">
      {#if e.kind == "separator"}
        <hr class="separator" part="separator">
      {:else if e.kind == "header"}
        <span class="header heading" part="heading">{e.label}</span>
      {:else}
        <button part="row" class="entry" class:danger={e.danger} tone={e.danger && !e.disabled ? 'danger' : ''} role={e.kind == "check" ? "menuitemcheckbox" : e.kind == "radio" ? "menuitemradio" : "menuitem"} aria-checked={e.kind == "check" || e.kind == "radio" ? e.checked : null} aria-label={e.label} disabled={e.disabled} onclick={choose}><div class="face" layout="row">
          <div class="slot cover" layout="stack" fill gap="none">
            {#if e.icon && e.kind != "check" && e.kind != "radio"}<og-icon class="glyph icon" part="icon" icon={e.icon}></og-icon>{/if}
            {#if e.kind == "check" && e.checked}<svg class="check" part="check" viewBox="0 0 12 12"><polyline part="glyph" points="2,6.5 5,9.5 10,3"></polyline></svg>{/if}
            {#if e.kind == "radio" && e.checked}<span class="dot" part="dot"></span>{/if}
          </div>
          <span class="name" fill>{e.label}</span>
          {#if e.shortcut}<span class="shortcut" part="caption">{e.shortcut}</span>{/if}
          {#if e.more}<og-icon class="chevron" part="chevron" icon="chevronRight"></og-icon>{/if}
        </div></button>
      {/if}
    </div>{/if}
  {/each}
</menu>

<style>
.cover { justify-content: center; align-items: center; }
/* A dense smoked plate with a lit top edge. A row is type, dim until it is
   the one under the pointer or the focus, then washed in the accent from a
   lit leading edge, as a menu entry is; a shortcut sits on a dark key
   chip; a check is drawn in the accent and the chosen radio is a lit
   diamond; a disabled row is faint and a destructive one in the danger
   ink. */
#Menu { min-inline-size: calc(240 * var(--font-step-px)); max-inline-size: min(100%, var(--measure-md)); margin: 0; padding: var(--space-xs) 0 var(--space-sm); list-style: none; border: 0; border-radius: 0; background: var(--dialog-fill); box-shadow: var(--dialog-menu-edge); color: var(--front-ink); }
.row { width: stretch; }
button.entry { --button-pad: 0; --button-pressed-pad: 0; --button-font-size: var(--dialog-row-size); --button-ink: var(--front-ink-dim); --button-hover-ink: var(--front-ink); --button-disabled-ink: var(--system-ink-off); --button-radius: 0; --button-focus-radius: 0; width: stretch; min-width: 0; height: auto; min-block-size: calc(38 * var(--font-step-px)); padding: var(--space-xs) var(--space-md) var(--space-xs) var(--space-lg); border-radius: 0; background: transparent; box-shadow: none; }
button.entry { padding: 0; border-radius: 0; background: transparent; box-shadow: none; color: var(--front-ink-dim); }
button.entry:hover, button.entry:focus-visible { background: var(--front-entry-on-fill); box-shadow: var(--front-entry-on-edge); color: var(--front-ink); }
button.entry:active { background: var(--front-entry-press-fill); box-shadow: var(--front-entry-on-edge); color: var(--front-ink); }
button.entry:disabled { background: transparent; box-shadow: none; color: var(--system-ink-off); }
button.entry.danger { --button-ink: var(--system-danger-ink); --button-hover-ink: var(--system-danger-ink); }
button.entry.danger, button.entry.danger:hover, button.entry.danger:focus-visible { color: var(--system-danger-ink); }
button.entry.danger:disabled { color: var(--system-ink-off); }
@media (pointer: coarse) { button.entry { min-block-size: var(--touch-target); } }
.face { width: stretch; column-gap: var(--space-sm); align-items: center; }
.slot { flex: none; width: calc(18 * var(--font-step-px)); height: calc(18 * var(--font-step-px)); }
.icon { width: 100%; height: 100%; }
.check { width: 75%; height: 75%; color: var(--front-accent); }
.dot { width: 45%; height: 45%; rotate: 45deg; border: 0; border-radius: 0; background: var(--front-accent); box-shadow: none; }
/* A long command wraps between its words; nothing is cut. */
.name { flex: 1 1 auto; min-inline-size: 0; text-align: start; font-size: var(--dialog-row-size); line-height: var(--font-leading-snug); font-weight: var(--dialog-weight); overflow-wrap: break-word; }
.shortcut { flex: none; padding: var(--space-xxs) var(--space-xs); font-size: var(--hud-tag-size); font-weight: var(--dialog-strong-weight); line-height: var(--font-leading-snug); letter-spacing: var(--dialog-chip-tracking); white-space: nowrap; color: var(--front-ink-dim); background: var(--hud-tag-fill); box-shadow: var(--dialog-chip-rim); }
button.entry:disabled .shortcut { color: var(--system-ink-off); }
/* The back and submenu marks are the engine's line chevrons, as wide as
   the menu's other icons. */
.chevron { flex: none; width: calc(16 * var(--font-step-px)); height: calc(16 * var(--font-step-px)); }
.chevron:dir(rtl) { scale: -1 1; }
.heading { padding: var(--space-sm) var(--space-lg) var(--space-xxs); font-size: var(--hud-label-size); font-weight: var(--dialog-strong-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--front-ink-faint); }
.heading:lang(zh, ja, ko) { font-size: var(--dialog-heading-cjk-size); letter-spacing: 0; }
.separator { width: auto; align-self: stretch; margin: var(--space-xs) var(--space-lg); color: var(--dialog-rule); }
</style>
