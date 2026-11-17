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
:host { inline-size: stretch; }
.menu-wrap { position: relative; }
.menu-scroll { overflow-x: auto; overflow-y: hidden; }
.menu-fade { display: none; position: absolute; inset-inline-end: 0; top: 0; bottom: 0; width: var(--size-lg); pointer-events: none; }
.menu-tabs { grid-template-columns: none; grid-auto-flow: column; grid-auto-columns: 1fr; }
[part="RpgMenuBar.tab"] { min-height: var(--size-touch); padding-inline: var(--space-md); }
.menu-close { width: var(--size-touch); height: var(--size-touch); flex: none; padding: 0; }
.face { inline-size: 100%; block-size: 100%; }
.close-mark { width: 50%; height: 50%; }
@media (width < 600px), (height < 500px) { :host { --layout-gap-loose: var(--layout-gap-tight); } .menu-tabs { grid-auto-columns: max-content; width: max-content; } .menu-fade { display: block; } }
</style>
