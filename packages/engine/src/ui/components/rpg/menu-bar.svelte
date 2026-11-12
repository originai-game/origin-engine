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
<nav id="RpgMenuBar" part="RpgMenuBar.frame" aria-label={t("rpg.menu.name")} ontabNext={tabNext} ontabPrev={tabPrev} onback={close}>
  <div class="menu-wrap">
    <scroll class="menu-scroll">
      <div class="menu-tabs" role="tablist">
        {#each tabs as c (c.id)}
          <button part="RpgMenuBar.tab" role="tab" aria-selected={c.id == tab} onclick={choosetab}>{c.label}</button>
        {/each}
      </div>
    </scroll>
    <div class="menu-fade"></div>
  </div>
  <button id="Close" class="menu-close" aria-label={t("rpg.menu.close")} onclick={close}><div class="face"><div class="close-mark"></div></div></button>
</nav>

<style>
[part="RpgMenuBar.frame"] { display: flex; width: stretch; align-items: center; gap: var(--space-md); }
.menu-wrap { position: relative; width: stretch; min-width: 0; }
.menu-scroll { width: stretch; overflow-x: auto; overflow-y: hidden; }
.menu-fade { display: none; position: absolute; left: calc(100% - var(--size-lg)); top: 0; width: var(--size-lg); height: stretch; pointer-events: none; }
.menu-fade:dir(rtl) { left: 0; }
.menu-tabs { display: grid; grid-auto-flow: column; grid-auto-columns: 1fr; gap: var(--space-xs); width: stretch; }
[part="RpgMenuBar.tab"] { width: stretch; min-height: var(--size-touch); }
.menu-close { width: var(--size-touch); height: var(--size-touch); flex: none; padding: 0; }
.face { width: stretch; height: stretch; align-items: center; justify-content: center; }
.close-mark { width: 50%; height: 50%; }
@media (width < 600px), (height < 500px) { [part="RpgMenuBar.frame"] { gap: var(--space-xs); } .menu-tabs { grid-auto-columns: max-content; width: max-content; } .menu-fade { display: block; } }
</style>
