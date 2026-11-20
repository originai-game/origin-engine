<meta name="component" content="Breadcrumbs">
<meta name="strings" content="./breadcrumbs.en.ogt">
<meta name="prop:items" content="list" data-doc="The crumbs, the current page last: each { id, label, icon, current }, current true on the last.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls it; empty is Breadcrumb.">
<meta name="action:choose" content="Go back">
<meta name="fixture:default" content='{"label":"","items":[{"id":"home","label":"Home","icon":"home","current":false},{"id":"armory","label":"Armory","icon":"","current":false},{"id":"swords","label":"Swords","icon":"","current":false},{"id":"ember","label":"Ember Blade","icon":"","current":true}]}'>

<nav id="Breadcrumbs" part="frame" variant="bare" aria-label={label || t("component.breadcrumbs.name")}>
  <ol id="Trail" layout="wrap" gap="tight">
    {#each items as crumb (crumb.id)}
      <li part="crumb" class="crumb" layout="row" gap="tight" class:current={crumb.current}>
        {#if crumb.icon}<og-icon part="icon" class="icon" icon={crumb.icon}></og-icon>{/if}
        {#if crumb.current}<span part="label" class="here current" aria-current="page">{crumb.label}</span>
        {:else}<a part="caption" class="link" href={crumb.id} onclick={choose}>{crumb.label}</a>
        <svg part="separator" class="sep" viewBox="0 0 8 12"><path part="glyph" d="M2 1l4 5-4 5"></path></svg>{/if}
      </li>
    {/each}
  </ol>
</nav>

<style>
/* A trail in type: dim links, faint chevrons, the current page bold on a
   lit underline. The trail wraps between crumbs, and a long current page
   takes a line of its own and wraps between its words. */
#Breadcrumbs { display: block; color: var(--front-ink); }
ol { margin: 0; padding: 0; list-style-type: none; align-items: center; row-gap: 0; }
.crumb { min-inline-size: 0; max-inline-size: 100%; min-block-size: calc(32 * var(--font-step-px)); align-items: center; }
.crumb:touch { min-block-size: var(--touch-target); }
.icon { flex: none; width: calc(18 * var(--font-step-px)); height: calc(18 * var(--font-step-px)); color: var(--front-ink-dim); }
.link { font-size: var(--dialog-crumb-size); font-weight: var(--dialog-weight); line-height: var(--font-leading-snug); color: var(--front-ink-dim); text-decoration: none; overflow-wrap: break-word; min-inline-size: 0; }
.link:hover, .link:focus-visible { color: var(--front-ink); }
.current { min-inline-size: 0; padding-block-end: var(--space-xxs); font-size: var(--dialog-crumb-size); font-weight: var(--dialog-strong-weight); line-height: var(--font-leading-snug); letter-spacing: 0; text-transform: none; color: var(--front-ink); border-block-end: var(--dialog-crumb-line); overflow-wrap: break-word; }
.sep { flex: none; width: calc(8 * var(--font-step-px)); height: calc(12 * var(--font-step-px)); color: var(--system-ink-off); }
.sep:dir(rtl) { scale: -1 1; }
</style>
