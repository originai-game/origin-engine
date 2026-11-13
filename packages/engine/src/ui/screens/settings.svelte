<meta name="screen" content="settings" data-layer="menu" data-focus="Tree" data-under="dim" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="back">
<meta name="strings" content="./settings.en.ogt">
<meta name="parts" content="settings.frame settings.band settings.title settings.row">
<meta name="prop:note" content="string" data-doc="A line under the title (Some changes apply after a restart); empty: none.">
<meta name="prop:pages" content="list" data-doc="The settings' pages as a keyed collection of { id, name }: a tab each.">
<meta name="prop:entries" content="list" data-doc="The settings as a keyed collection of { id, page, name, description, kind: toggle|range|select|binding, value, min, max, step, options, action }, shown on their page's tab; each value is written back as the player changes it.">
<meta name="prop:page" content="string" data-doc="The id of the page shown (written back as the player picks a tab); empty: no tree of pages, the `tree` content alone.">
<meta name="prop:pending" content="bool" data-doc="Changes wait to be applied: Apply and Revert take presses.">
<meta name="action:confirm" content="Select">
<meta name="action:back" content="Back">
<meta name="action:tabNext" content="Next page">
<meta name="action:tabPrev" content="Previous page">
<meta name="fixture:default" content="{&quot;note&quot;:&quot;&quot;}">

<column id="Screen" part="settings.frame" class="bare">
  <row id="Header" part="settings.band">
    <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
    <column id="Heading">
      <text id="Title" part="settings.title" heading="true">{t("screen.settings.title")}</text>
      <text id="Note" hidden={!(($note != ""))}>{$note}</text>
    </column>
  </row>
  <column id="Tree">
    {#if $page}
      <nav id="Pages" role="tablist">
        {#each $pages as p (p.id)}
          <label class="tab"><input type="radio" id="Tab" value={p.id} bind:group={$page}>{p.name}</label>
        {/each}
      </nav>
      <ul id="Entries">
        {#each $entries as e (e.id)}
          {#if e.page == $page}
            <li class="entry" part="settings.row" title={e.description}>
              {#if e.kind == "toggle"}
                <label class="row">{e.name}<input type="checkbox" switch id="Toggle" bind:checked={e.value}></label>
              {/if}
              {#if e.kind == "range"}
                <p class="name">{e.name}</p>
                <input type="range" id="Range" aria-label={e.name} min={e.min} max={e.max} step={e.step} bind:value={e.value}>
              {/if}
              {#if e.kind == "select"}
                <p class="name">{e.name}</p>
                <select id="Choice" data-cycle aria-label={e.name} items={e.options} bind:value={e.value}></select>
              {/if}
              {#if e.kind == "binding"}
                <p class="name">{e.name}</p>
                <kbd id="Binding" data-action={e.action} data-og-rebind bind:value={e.value}></kbd>
              {/if}
            </li>
          {/if}
        {/each}
      </ul>
      <footer id="Foot">
        <button id="Defaults" variant="quiet" onclick={defaults}>{t("screen.settings.defaults")}</button>
        <button id="Revert" disabled={!$pending} onclick={revert}>{t("screen.settings.revert")}</button>
        <button id="Apply" variant="primary" disabled={!$pending} onclick={apply}>{t("screen.settings.apply")}</button>
      </footer>
    {/if}
    {@render tree?.()}
  </column>
</column>

<style>
#Screen { width: stretch; height: stretch; padding: var(--space-xxl); gap: var(--space-lg); }
@media (width < 600px), (height < 600px) {
#Screen { padding: var(--space-lg); gap: var(--space-md); }
}
#Header { width: stretch; gap: var(--space-md); align-items: center; }
#Heading { width: stretch; gap: var(--space-xxs); }
#Title { font-size: var(--type-xl2-size); }
@media (width < 600px), (height < 600px) {
#Title { font-size: var(--type-xl-size); }
}
#Note { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); line-clamp: 2; }
#Tree { width: stretch; height: stretch; gap: var(--space-md); }
#Pages { display: flex; gap: var(--space-sm); flex-wrap: wrap; }
#Entries { list-style: none; padding: 0; margin: 0; width: stretch; height: stretch; overflow-y: auto; }
.entry { display: flex; align-items: center; gap: var(--space-md); padding: var(--space-sm) 0; border-bottom: var(--stroke-hairline) solid var(--line); }
.entry .name { width: stretch; }
#Foot { display: flex; justify-content: end; gap: var(--space-sm); width: stretch; }
</style>
