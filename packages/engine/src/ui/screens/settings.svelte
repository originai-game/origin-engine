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
<meta name="fixture:default" content='{"note":"Some changes apply after a restart.","pages":[{"id":"general","name":"General"}],"page":"general","pending":true,"entries":[{"id":"subtitles","page":"general","name":"Subtitles","description":"Show dialogue captions.","kind":"toggle","value":true},{"id":"volume","page":"general","name":"Master volume","description":"Adjust all game audio.","kind":"range","value":75,"min":0,"max":100,"step":5},{"id":"quality","page":"general","name":"Quality","description":"Choose the rendering quality.","kind":"select","value":"High","options":["Low","High"]},{"id":"interact","page":"general","name":"Interact","description":"Choose the interaction key.","kind":"binding","value":"E","action":"interact"}]}'>

<div id="Screen" part="settings.frame" class="bare">
  <header id="Header" part="settings.band">
    <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
    <div id="Heading">
      <h1 id="Title" part="settings.title">{t("screen.settings.title")}</h1>
      {#if note != ""}<p id="Note">{note}</p>{/if}
    </div>
  </header>
  <div id="Tree">
    {#if page}
      <nav id="Pages" role="tablist">
        {#each pages as p (p.id)}
          <label class="tab"><input type="radio" id="Tab" value={p.id} bind:group={page}>{p.name}</label>
        {/each}
      </nav>
      <ul id="Entries">
        {#each entries as e (e.id)}
          {#if e.page == page}
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
      <footer id="Foot" data-layout="cluster">
        <button id="Defaults" variant="quiet" onclick={defaults}>{t("screen.settings.defaults")}</button>
        <button id="Revert" disabled={!pending} onclick={revert}>{t("screen.settings.revert")}</button>
        <button id="Apply" variant="primary" disabled={!pending} onclick={apply}>{t("screen.settings.apply")}</button>
      </footer>
    {/if}
    {@render tree?.()}
  </div>
</div>

<style>
:root { display: flex; flex-direction: column; gap: var(--space-lg); block-size: stretch; padding: var(--space-xxl); }
h1, #Note { margin: 0; }
[part="settings.band"] { display: flex; align-items: center; gap: var(--space-md); }
#Heading { flex: 1; min-inline-size: 0; display: flex; flex-direction: column; gap: var(--space-xxs); }
[part="settings.title"] { font-size: var(--type-xl2-size); letter-spacing: var(--type-xl2-tracking); }
#Note { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); line-clamp: 2; }
#Tree { flex: 1; min-block-size: 0; display: flex; flex-direction: column; gap: var(--space-md); }
#Pages { display: flex; flex-wrap: wrap; gap: var(--space-sm); }
#Entries { flex: 1; min-block-size: 0; overflow-y: auto; margin: 0; padding: 0; list-style: none; }
.entry { display: flex; align-items: center; gap: var(--space-md); padding-block: var(--space-sm); border-block-end: var(--stroke-hairline) solid var(--line); }
.entry .name { flex: 1; min-inline-size: 0; }
#Foot { justify-content: end; gap: var(--space-sm); }
@media (width < 600px), (height < 600px) {
  :root { padding: var(--space-lg); gap: var(--space-md); }
  [part="settings.title"] { font-size: var(--type-xl-size); letter-spacing: var(--type-xl-tracking); }
}
</style>
