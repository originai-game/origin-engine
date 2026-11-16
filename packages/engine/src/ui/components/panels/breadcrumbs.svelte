<meta name="component" content="Breadcrumbs">
<meta name="strings" content="./breadcrumbs.en.ogt">
<meta name="parts" content="Breadcrumbs.frame Breadcrumbs.crumb Breadcrumbs.icon Breadcrumbs.link Breadcrumbs.current Breadcrumbs.separator">
<meta name="prop:items" content="list" data-doc="The crumbs, the current page last: each { id, label, icon, current }, current true on the last.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls it; empty is Breadcrumb.">
<meta name="action:choose" content="Go back">
<meta name="fixture:default" content='{"label":"","items":[{"id":"home","label":"Home","icon":"home","current":false},{"id":"armory","label":"Armory","icon":"","current":false},{"id":"swords","label":"Swords","icon":"","current":false},{"id":"ember","label":"Ember Blade","icon":"","current":true}]}'>

<nav id="Breadcrumbs" part="Breadcrumbs.frame" class="bare" aria-label={label || t("component.breadcrumbs.name")}>
  <ol id="Trail">
    {#each items as crumb (crumb.id)}
      <li part="Breadcrumbs.crumb" class="crumb" class:current={crumb.current}>
        {#if crumb.icon}<og-icon part="Breadcrumbs.icon" class="icon" icon={crumb.icon}></og-icon>{/if}
        {#if crumb.current}<span part="Breadcrumbs.current" class="here" aria-current="page">{crumb.label}</span>
        {:else}<a part="Breadcrumbs.link" class="link" href={crumb.id} onclick={choose}>{crumb.label}</a>
        <svg part="Breadcrumbs.separator" class="sep" viewBox="0 0 8 12"><path d="M2 1l4 5-4 5" fill="none" stroke-width="1.5" stroke-linecap="round"></path></svg>{/if}
      </li>
    {/each}
  </ol>
</nav>

<style>
:host { display: block; }
ol { margin: 0; padding: 0; list-style-type: none; }
#Trail { display: flex; gap: var(--space-xs); align-items: center; flex-wrap: wrap; }
.crumb { display: flex; gap: var(--space-xs); align-items: center; }
.icon { width: var(--size-xs); height: var(--size-xs); color: var(--ink-muted); }
.link { font: var(--text-sm); color: var(--ink-muted); }
.link:hover { color: var(--ink); }
.here { font: var(--text-sm); color: var(--ink); }
.sep { width: var(--space-sm); height: var(--space-md); stroke: var(--ink-faint); }
.sep:dir(rtl) { scale: -1 1; }
</style>
