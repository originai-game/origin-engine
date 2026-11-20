<meta name="component" content="RpgMenuBar">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgMenuBar.frame RpgMenuBar.tab">
<meta name="prop:tabs" content="list" data-doc="The menu's screens, keyed rows of { id, label } (Character, Inventory, Skills, Quests, Map).">
<meta name="prop:tab" content="string" data-doc="The screen shown.">
<meta name="action:tabPrev" content="Previous screen">
<meta name="action:tabNext" content="Next screen">
<meta name="action:back" content="Close">
<meta name="fixture:default" content='{"tabs":[{"id":"char","label":"Character"},{"id":"inv","label":"Inventory"}],"tab":"char"}'>

<!--
  The menu's top bar: its screens as tabs and a close button. Reports tab
  (a screen's key), tabPrev, tabNext and close (the button, Escape or B).
-->
<nav id="RpgMenuBar" part="RpgMenuBar.frame" class="bare" data-layout="row" data-gap="loose" aria-label={t("rpg.menu.name")} ontabNext={tabNext} ontabPrev={tabPrev} onback={close}>
  <div class="menu-wrap" data-fill>
    <div class="menu-scroll">
      <div class="menu-tabs" data-layout="grid" data-gap="tight" role="tablist">
        {#each tabs as c (c.id)}
          <button part="RpgMenuBar.tab" role="tab" aria-selected={c.id == tab} onclick={choosetab}>{c.label}</button>
        {/each}
      </div>
    </div>
    <div class="menu-fade"></div>
  </div>
  <button id="Close" class="menu-close" aria-label={t("rpg.menu.close")} onclick={close}><div class="face" data-layout="cover"><og-icon class="close-mark" icon="close"></og-icon></div></button>
</nav>

<style>
/* The menu's top bar: its screens as tracked capitals over a rule, the
   chosen one washed and underlined in the accent; Close a cut plate. */
:host { inline-size: stretch; color: var(--hud-ink); }
.menu-wrap { position: relative; min-width: 0; box-shadow: var(--party-rule); }
.menu-scroll { overflow-x: auto; overflow-y: hidden; }
.menu-fade { display: none; position: absolute; inset-inline-end: 0; top: 0; bottom: 0; width: var(--size-lg); pointer-events: none; }
.menu-tabs { grid-template-columns: none; grid-auto-flow: column; grid-auto-columns: 1fr; --layout-gap-tight: 0px; }
[part="RpgMenuBar.tab"] { min-height: var(--size-touch); padding-inline: var(--space-lg); border: 0; border-radius: 0; white-space: nowrap; font-size: var(--hud-size-sm); font-weight: var(--hud-figure-weight); letter-spacing: var(--result-tab-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink-dim); background: none; box-shadow: none; }
[part="RpgMenuBar.tab"]:hover { background: var(--result-tab-hover); }
[part="RpgMenuBar.tab"][aria-selected=true] { color: var(--hud-ink); background: var(--result-tab-selected); box-shadow: var(--result-tab-mark); }
[part="RpgMenuBar.tab"]:focus-visible { box-shadow: var(--party-focus-rim); }
.menu-close { width: var(--size-touch); height: var(--size-touch); flex: none; padding: 0; border: 0; color: var(--hud-ink); background: var(--party-button-fill); box-shadow: var(--party-button-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.menu-close:hover { background: var(--party-button-hover-fill); }
.menu-close:focus-visible { box-shadow: var(--party-focus-rim); }
.face { inline-size: 100%; block-size: 100%; }
.close-mark { width: 50%; height: 50%; }
@media (width < 600px), (height < 500px) { :host { --layout-gap-loose: var(--layout-gap-tight); } .menu-tabs { grid-auto-columns: max-content; width: max-content; } .menu-fade { display: block; } [part="RpgMenuBar.tab"] { padding-inline: var(--space-md); font-size: var(--hud-size-xs); letter-spacing: var(--result-tab-tracking-compact); } }
</style>
