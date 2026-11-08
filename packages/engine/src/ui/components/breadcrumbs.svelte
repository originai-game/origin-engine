<meta name="component" content="Breadcrumbs">
<meta name="strings" content="./breadcrumbs.en.ogt">
<meta name="prop:items" content="list" data-doc="The crumbs, the current page last: each { id, label, icon, current }, current true on the last.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls it; empty is Breadcrumb.">
<meta name="action:choose" content="Go back">
<meta name="fixture:default" content='{"label":"","items":[{"id":"home","label":"Home","icon":"home","current":false},{"id":"armory","label":"Armory","icon":"","current":false},{"id":"swords","label":"Swords","icon":"","current":false},{"id":"ember","label":"Ember Blade","icon":"","current":true}]}'>

<nav id="Breadcrumbs" aria-label={label || t("component.breadcrumbs.name")}>
  <ol id="Trail">
    {#each items as crumb (crumb.id)}
      <li class="crumb" class:current={crumb.current}>
        <icon class="icon" icon={crumb.icon} hidden={!crumb.icon}></icon>
        <a class="link" href={crumb.id} aria-current={crumb.current ? 'page' : ''} onclick={choose} hidden={crumb.current}>{crumb.label}</a>
        <span class="here" hidden={!crumb.current}>{crumb.label}</span>
        <svg class="sep" viewBox="0 0 8 12" hidden={crumb.current}><path d="M2 1l4 5-4 5" fill="none" stroke-width="1.5" stroke-linecap="round"></path></svg>
      </li>
    {/each}
  </ol>
</nav>

<style>
ul, ol { list-style-type: none; }
#Trail { display: flex; gap: 6px; align-items: center; flex-wrap: wrap; }
.crumb { display: flex; gap: 6px; align-items: center; }
.icon { width: 16px; height: 16px; color: var(--ink-muted); }
.link { font: var(--text-sm); color: var(--ink-muted); }
.link:hover { color: var(--ink); }
.here { font: var(--text-sm); color: var(--ink); }
.sep { width: 8px; height: 12px; stroke: var(--ink-faint); }
</style>
