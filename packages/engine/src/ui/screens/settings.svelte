<meta name="screen" content="settings" data-layer="menu" data-focus="Tree" data-under="dim" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="back">
<meta name="strings" content="./settings.en.ogt">
<meta name="parts" content="settings.frame settings.band settings.title settings.row settings.shade settings.tabs settings.tab settings.description">
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
  <div id="Veil" part="settings.shade"></div>
  <header id="Header" part="settings.band">
    <button id="Back" variant="quiet" onclick={back}><div class="back"><og-icon class="chevron" icon="chevronLeft"></og-icon><span>{t("screen.common.back")}</span></div></button>
    <div id="Heading">
      <h1 id="Title" part="settings.title">{t("screen.settings.title")}</h1>
      {#if note != ""}<p id="Note">{note}</p>{/if}
    </div>
  </header>
  <div id="Tree">
    {#if page}
      <nav id="Pages" part="settings.tabs" role="tablist">
        <kbd id="Prev" class="cap" data-action="tabPrev"></kbd>
        <div id="Strip">
          {#each pages as p (p.id)}
            <label class="tab" part="settings.tab"><input type="radio" id="Tab" value={p.id} bind:group={page}>{p.name}</label>
          {/each}
        </div>
        <kbd id="Next" class="cap" data-action="tabNext"></kbd>
      </nav>
      <div id="Body">
        <ul id="Entries">
          {#each entries as e (e.id)}
            {#if e.page == page}
              <li class="entry" part="settings.row" title={e.description}>
                {#if e.kind == "toggle"}
                  <label class="row name">{e.name}<input type="checkbox" switch id="Toggle" bind:checked={e.value}></label>
                {/if}
                {#if e.kind == "range"}
                  <div class="row"><p class="name">{e.name}</p><input type="range" id="Range" aria-label={e.name} min={e.min} max={e.max} step={e.step} bind:value={e.value}></div>
                {/if}
                {#if e.kind == "select"}
                  <div class="row"><p class="name">{e.name}</p><select id="Choice" data-cycle aria-label={e.name} items={e.options} bind:value={e.value}></select></div>
                {/if}
                {#if e.kind == "binding"}
                  <div class="row"><p class="name">{e.name}</p><kbd id="Binding" data-action={e.action} data-og-rebind bind:value={e.value}></kbd></div>
                {/if}
                {#if e.description != ""}<p class="description" part="settings.description">{e.description}</p>{/if}
              </li>
            {/if}
          {/each}
        </ul>
      </div>
      <footer id="Foot">
        <button id="Defaults" variant="quiet" onclick={defaults}>{t("screen.settings.defaults")}</button>
        <button id="Revert" class="plate" onclick={revert} disabled={!pending}>{t("screen.settings.revert")}</button>
        <button id="Apply" variant="primary" class="plate" disabled={!pending} onclick={apply}>{t("screen.settings.apply")}</button>
      </footer>
    {/if}
    {@render tree?.()}
  </div>
</div>

<style>
/* Over the dimmed game: a veil; the band (Back, the title, the note); the
   pages as a strip of tabs in type, the chosen one on a lit bar, which
   scrolls sideways and fades at its edges when the pages outrun it; the
   settings as glass strips, the focused one lit like a menu entry; its
   description beside the list on a wide screen and under its row on a
   narrow one; Revert and Apply as cut plates. */
:root { position: relative; display: flex; flex-direction: column; row-gap: var(--space-lg); block-size: stretch; overflow: hidden; padding: var(--space-xxl) var(--space-3xl) var(--space-xl); color: var(--front-ink); }
[part="settings.shade"] { position: absolute; inset: 0; background: var(--system-veil); }
[part="settings.band"], #Tree { position: relative; }
h1, p { margin: 0; }
[part="settings.band"] { display: flex; align-items: baseline; column-gap: var(--space-lg); }
#Back { --button-ink: var(--front-ink-dim); --button-quiet-ink: var(--front-ink-dim); --button-quiet-hover-ink: var(--front-ink); }
.back { display: flex; align-items: center; column-gap: var(--space-xs); }
.back .chevron { inline-size: var(--size-xs); block-size: var(--size-xs); }
.chevron:dir(rtl) { scale: -1 1; }
#Heading { flex: 1; min-inline-size: 0; display: flex; align-items: baseline; flex-wrap: wrap; column-gap: var(--space-lg); row-gap: var(--space-xxs); }
[part="settings.title"] { font-size: var(--front-title-size); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); font-stretch: var(--front-stretch); letter-spacing: var(--font-tracking-xs); text-shadow: var(--front-shadow); }
#Note { margin-inline-start: auto; font-size: calc(14 * var(--font-step-px)); color: var(--front-lock-ink); line-clamp: 2; }
#Tree { flex: 1; min-block-size: 0; display: flex; flex-direction: column; row-gap: var(--space-lg); }
/* The tabs. */
[part="settings.tabs"] { display: flex; align-items: center; column-gap: var(--space-md); border-block-end: var(--stroke-hairline) solid var(--system-tabs-rule); }
.cap { flex: none; }
#Strip { flex: 1 1 auto; min-inline-size: 0; display: flex; column-gap: var(--space-xl); overflow: auto hidden; scrollbar-width: none; padding-block: var(--space-xxs); mask-image: linear-gradient(90deg, transparent, #000 var(--space-lg), #000 calc(100% - var(--space-3xl)), transparent); padding-inline: var(--space-lg); }
/* A tab is a radio drawn as a word: no ring, the chosen one inked and
   underlined by a lit bar. */
[part="settings.tab"] { flex: none; }
#Tab::part(ring), #Tab::part(dot) { inline-size: 0; block-size: 0; min-inline-size: 0; margin: 0; border-width: 0; background: none; box-shadow: none; }
#Tab::part(chosen) { border-width: 0; background: none; }
#Tab::part(option) { column-gap: 0; min-block-size: var(--touch-target); padding: 0 var(--space-xxs); border-radius: 0; background: none; font-size: calc(17 * var(--font-step-px)); font-weight: var(--font-weight-semibold); letter-spacing: var(--hud-label-tracking); text-transform: uppercase; white-space: nowrap; color: var(--front-ink-faint); }
#Tab::part(option):hover { background: none; color: var(--front-ink-dim); }
#Tab::part(option):checked { color: var(--front-ink); box-shadow: inset 0 calc(-3 * var(--font-step-px)) 0 var(--system-tab-bar); }
/* The rows; the focused row's description in the room on the right. */
#Body { position: relative; flex: 1; min-block-size: 0; display: grid; grid-template-columns: minmax(0, 3fr) minmax(0, 2fr); grid-template-rows: minmax(0, 1fr); column-gap: var(--space-3xl); }
#Entries { flex: 1; min-inline-size: 0; display: flex; flex-direction: column; row-gap: var(--space-xxs); margin: 0; padding: 0; list-style: none; overflow: hidden auto; }
[part="settings.row"] { flex: none; display: flex; flex-direction: column; border-radius: 0; background: var(--system-row-fill); box-shadow: var(--system-row-rim); }
[part="settings.row"]:hover, [part="settings.row"]:focus-within { background: var(--system-row-on-fill); box-shadow: var(--system-row-on-edge); }
div.row { display: grid; grid-template-columns: minmax(min(calc(160 * var(--font-step-px)), 100%), 1fr) minmax(0, calc(320 * var(--font-step-px))); align-items: center; column-gap: var(--space-xl); }
.row { min-block-size: calc(56 * var(--font-step-px)); padding: var(--space-xs) var(--space-lg); font-size: calc(18 * var(--font-step-px)); }
.name { flex: 1 1 auto; min-inline-size: 0; color: var(--front-ink-dim); overflow-wrap: break-word; }
[part="settings.row"]:focus-within .name, [part="settings.row"]:hover .name { color: var(--front-ink); }
#Range, #Choice { min-inline-size: 0; }
#Binding { justify-self: end; }
/* A toggle row is the switch's own label: its text starts where the
   other rows' names do. */
#Toggle::part(row) { padding-inline: 0; }
[part="settings.description"] { display: none; }
[part="settings.row"]:focus-within [part="settings.description"] { display: block; position: absolute; inset-block-start: 0; inset-inline-end: 0; inline-size: calc((100% - var(--space-3xl)) * 0.4); padding: var(--space-lg) var(--space-xl); font-size: calc(15 * var(--font-step-px)); line-height: var(--font-leading-normal); color: var(--front-ink-dim); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(var(--front-cut) 0, 100% 0, 100% calc(100% - var(--front-cut)), calc(100% - var(--front-cut)) 100%, 0 100%, 0 var(--front-cut)); }
/* The foot. */
#Foot { display: flex; flex-wrap: wrap; align-items: center; justify-content: end; gap: var(--space-sm) var(--space-md); }
#Defaults { margin-inline-end: auto; }
button.plate { --button-radius: 0; --button-focus-radius: 0; --button-border-width: 0; --button-font-size: calc(16 * var(--font-step-px)); --button-tracking: var(--font-tracking-md); --button-transform: uppercase; min-inline-size: calc(160 * var(--font-step-px)); }
button.plate, button.plate::part(base) { block-size: var(--control-lg); }
button.plate::part(base) { border-width: 0; clip-path: polygon(var(--front-cut) 0, 100% 0, 100% calc(100% - var(--front-cut)), calc(100% - var(--front-cut)) 100%, 0 100%, 0 var(--front-cut)); }
#Apply { --button-primary-fill: var(--front-play-fill); --button-primary-hover-fill: var(--front-play-hover-fill); --button-primary-pressed-fill: var(--front-play-pressed-fill); --button-primary-ink: var(--front-play-ink); --button-primary-hover-ink: var(--front-play-ink); --button-primary-pressed-ink: var(--front-play-ink); --button-primary-shadow: none; --button-primary-rim: transparent; --button-primary-hover-rim: transparent; }
#Revert { --button-fill: var(--hud-plate-fill); --button-hover-fill: var(--hud-plate-fill); --button-base-shadow: var(--system-plate-edge); --button-hover-shadow: var(--system-plate-lit-edge); --button-rim: transparent; --button-hover-rim: transparent; --button-ink: var(--front-ink); --button-hover-ink: var(--front-ink); }
@media (pointer: coarse) { .cap { display: none; } }
/* A tablet or a phone: no side room; the focused row's description opens
   under its row. */
@media (width < 1024px) {
  #Body { grid-template-columns: minmax(0, 1fr); }
  [part="settings.row"]:focus-within [part="settings.description"] { position: static; inline-size: auto; padding: 0 var(--space-lg) var(--space-sm); font-size: calc(14 * var(--font-step-px)); background: none; box-shadow: none; clip-path: none; }
}
@media (width < 600px), (height < 600px) {
  :root { row-gap: var(--space-md); padding: var(--space-lg) var(--space-lg) var(--space-md); }
  [part="settings.band"] { flex-wrap: wrap; row-gap: var(--space-xs); }
  #Back { margin-inline-start: calc(var(--space-sm) * -1); }
  #Heading { flex: 1 0 100%; flex-direction: column; align-items: start; }
  [part="settings.title"] { font-size: var(--front-title-size-compact); }
  #Note { margin-inline-start: 0; font-size: calc(13 * var(--font-step-px)); }
  #Strip { column-gap: var(--space-lg); margin-inline: calc(var(--space-lg) * -1); }
  #Tab::part(option) { font-size: calc(14 * var(--font-step-px)); }
  .row { row-gap: var(--space-xs); padding: var(--space-sm) var(--space-md); font-size: calc(16 * var(--font-step-px)); }
  div.row { grid-template-columns: minmax(0, 1fr); align-items: stretch; }
  div.row:has(#Binding) { grid-template-columns: minmax(0, 1fr) auto; align-items: center; }
  /* Defaults on its own line, the two plates sharing the row under it. */
  #Foot { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); }
  #Defaults { grid-column: 1 / -1; justify-self: start; margin-inline: calc(var(--space-sm) * -1) 0; }
  button.plate { min-inline-size: 0; }
}
</style>
