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
<nav id="RpgMenuBar" part="RpgMenuBar.frame" class="bare" aria-label={t("rpg.menu.name")} ontabNext={tabNext} ontabPrev={tabPrev} onback={close}>
  <div class="menu-wrap">
    <div class="menu-scroll">
      <div class="menu-tabs" role="tablist">
        {#each tabs as c (c.id)}
          <button part="RpgMenuBar.tab" role="tab" aria-selected={c.id == tab} onclick={choosetab}>{c.label}</button>
        {/each}
      </div>
    </div>
    <div class="menu-fade"></div>
  </div>
  <button id="Close" class="menu-close" aria-label={t("rpg.menu.close")} onclick={close}><div class="face"><og-icon class="close-mark" icon="close"></og-icon></div></button>
</nav>

<style>
:host { display: flex; inline-size: stretch; align-items: center; gap: var(--space-md); }
.menu-wrap { position: relative; flex: 1; min-width: 0; }
.menu-scroll { overflow-x: auto; overflow-y: hidden; }
.menu-fade { display: none; position: absolute; inset-inline-end: 0; top: 0; bottom: 0; width: var(--size-lg); pointer-events: none; }
.menu-tabs { display: grid; grid-auto-flow: column; grid-auto-columns: 1fr; gap: var(--space-xs); }
[part="RpgMenuBar.tab"] { min-height: var(--size-touch); padding-inline: var(--space-md); }
[part="RpgMenuBar.tab"][aria-selected=true] { background: var(--selection); box-shadow: inset 0 calc(var(--stroke-thick) * -1) 0 var(--primary); }
.menu-close { width: var(--size-touch); height: var(--size-touch); flex: none; padding: 0; }
.face { display: flex; flex-direction: column; inline-size: 100%; block-size: 100%; align-items: center; justify-content: center; }
.close-mark { width: 50%; height: 50%; }
@media (width < 600px), (height < 500px) { :host { gap: var(--space-xs); } .menu-tabs { grid-auto-columns: max-content; width: max-content; } .menu-fade { display: block; } }
</style>
