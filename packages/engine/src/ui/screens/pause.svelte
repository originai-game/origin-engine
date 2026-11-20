<meta name="screen" content="pause" data-layer="menu" data-focus="Panel/Menu/Resume" data-under="blur" data-input="menu" data-pause="true" data-capture="true" data-remember="true" data-back="resume">
<meta name="strings" content="./pause.en.ogt">
<meta name="parts" content="pause.frame pause.title pause.objective pause.shade pause.status pause.entry">
<meta name="prop:objective" content="string" data-doc="The current objective (empty: none).">
<meta name="prop:played" content="number" data-doc="Seconds played this run.">
<meta name="prop:canSave" content="bool" data-doc="Saving is allowed here.">
<meta name="action:confirm" content="Select">
<meta name="action:back" content="Resume">
<meta name="fixture:default" content="{&quot;canSave&quot;:true,&quot;objective&quot;:&quot;Reach the lighthouse&quot;,&quot;played&quot;:4520}">

<div id="Pause">
  <div id="Shade" part="pause.shade"></div>
  <section id="Panel" part="pause.frame" class="bare">
    <h2 id="Title" part="pause.title">{t("screen.pause.title")}</h2>
    <div id="Status" part="pause.status">
      {#if objective != ""}
        <p id="Label">{t("screen.pause.objectiveLabel")}</p>
        <div id="Goal"><span class="mark"></span><p id="Objective" part="pause.objective">{objective}</p></div>
      {/if}
      <p id="Played">{t("screen.pause.Played", { played: formatDuration(played) })}</p>
    </div>
    <div id="Menu" focus-group="wrap">
      <button id="Resume" part="pause.entry" class="entry" onclick={resume}><div class="face"><span class="label">{t("screen.pause.resume")}</span><og-icon class="chevron" icon="chevronRight"></og-icon></div></button>
      <button id="Settings" part="pause.entry" class="entry" onclick={settings}><div class="face"><span class="label">{t("screen.pause.settings")}</span><og-icon class="chevron" icon="chevronRight"></og-icon></div></button>
      <button id="Controls" part="pause.entry" class="entry" onclick={controls}><div class="face"><span class="label">{t("screen.pause.controls")}</span><og-icon class="chevron" icon="chevronRight"></og-icon></div></button>
      <button id="Save" part="pause.entry" class="entry" disabled={!canSave} onclick={save}><div class="face"><span class="label">{t("screen.pause.save")}</span><og-icon class="chevron" icon="chevronRight"></og-icon></div></button>
      <button id="Load" part="pause.entry" class="entry" onclick={load}><div class="face"><span class="label">{t("screen.pause.load")}</span><og-icon class="chevron" icon="chevronRight"></og-icon></div></button>
      <hr id="Divider">
      <button id="QuitToMenu" part="pause.entry" class="entry minor" onclick={quitToMenu}><div class="face"><span class="label">{t("screen.pause.quitToMenu")}</span></div></button>
      <button id="Quit" part="pause.entry" class="entry minor danger" onclick={quit}><div class="face"><span class="label">{t("screen.pause.quit")}</span></div></button>
    </div>
  </section>
  <aside id="Side" class="side">{@render side?.()}</aside>
</div>

<style>
/* The game stays in view under a shade from the leading side; the menu is
   a column of type down that side, the run's status in a cut plate above
   it. Taller than the screen (a short phone, a large text scale) the
   column starts at the top and scrolls. */
:root { position: relative; display: flex; block-size: stretch; overflow: hidden; color: var(--front-ink); }
[part="pause.shade"] { position: absolute; inset: 0; background: var(--system-pause-shade); }
p { margin: 0; }
[part="pause.frame"] { position: relative; display: flex; flex-direction: column; inline-size: calc(560 * var(--font-step-px)); max-inline-size: 100%; block-size: 100%; padding: var(--space-xxl) var(--space-xxl) var(--space-xxl) var(--space-3xl); overflow: hidden auto; scrollbar-width: thin; transition: opacity var(--motion-enter), translate var(--motion-enter); }
[part="pause.frame"] > * { flex-shrink: 0; }
@starting-style { [part="pause.frame"] { opacity: 0; translate: calc(var(--space-xl) * -1) 0; } }
[part="pause.title"] { margin: 0; font-size: var(--system-title-size); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); font-stretch: var(--front-stretch); letter-spacing: var(--font-tracking-xs); text-transform: uppercase; text-shadow: var(--front-shadow); overflow-wrap: anywhere; }
[part="pause.status"] { display: flex; flex-direction: column; row-gap: var(--space-xxs); align-self: start; inline-size: calc(380 * var(--font-step-px)); max-inline-size: 100%; margin-block-start: var(--space-lg); padding: var(--space-md) var(--space-lg); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--front-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
#Label { font-size: calc(11 * var(--font-step-px)); font-weight: var(--font-weight-bold); letter-spacing: var(--front-heading-tracking); text-transform: uppercase; color: var(--front-ink-faint); }
#Goal { display: flex; align-items: center; column-gap: var(--space-sm); }
.mark { flex: none; inline-size: calc(8 * var(--font-step-px)); block-size: calc(8 * var(--font-step-px)); rotate: 45deg; background: var(--front-accent); box-shadow: var(--front-rule-glow); }
[part="pause.objective"] { flex: 1 1 auto; min-inline-size: 0; font-size: calc(18 * var(--font-step-px)); line-height: var(--font-leading-snug); font-weight: var(--font-weight-semibold); line-clamp: 2; }
#Played { margin-block-start: var(--space-xxs); font-size: calc(14 * var(--font-step-px)); font-variant-numeric: tabular-nums; color: var(--front-ink-dim); }
#Menu { display: flex; flex-direction: column; row-gap: var(--space-xxs); margin-block-start: auto; margin-inline-start: calc(var(--space-lg) * -1); padding-block-start: var(--space-xl); }
/* An entry is type, as on the main menu: dim until it is the one under the
   pointer or the focus, then washed in the accent from a lit leading edge. */
button.entry { --button-font-size: var(--system-entry-size); --button-text-shadow: var(--front-shadow); --button-ink: var(--front-ink-dim); --button-hover-ink: var(--front-ink); --button-disabled-ink: var(--system-ink-off); inline-size: stretch; min-block-size: var(--touch-target); background: transparent; box-shadow: none; border-radius: 0; }
button.entry::part(base) { background: transparent; box-shadow: none; border-radius: 0; padding: var(--space-xs) var(--space-lg); color: var(--front-ink-dim); }
button.entry:enabled::part(base):hover, button.entry:focus-visible::part(base) { background: var(--front-entry-on-fill); box-shadow: var(--front-entry-on-edge); color: var(--front-ink); }
button.entry:enabled::part(base):active { background: var(--front-entry-press-fill); box-shadow: var(--front-entry-on-edge); color: var(--front-ink); }
button.entry:disabled::part(base) { background: transparent; color: var(--system-ink-off); }
.face { display: flex; align-items: center; column-gap: var(--space-md); inline-size: stretch; }
.label { flex: 1 1 auto; min-inline-size: 0; text-align: start; font-size: var(--system-entry-size); line-height: var(--font-leading-tight); font-weight: var(--front-entry-weight); text-shadow: var(--front-shadow); overflow-wrap: anywhere; }
.chevron { flex: none; inline-size: var(--size-xs); block-size: var(--size-xs); color: var(--front-accent); opacity: 0; }
button.entry:hover .chevron, button.entry:focus-visible .chevron { opacity: 1; }
.chevron:dir(rtl) { scale: -1 1; }
#Divider { block-size: var(--stroke-hairline); margin: var(--space-sm) 0 var(--space-xs) var(--space-lg); border: 0; background: var(--front-divider); }
button.entry.minor { --button-font-size: var(--front-entry-quiet-size); --button-ink: var(--front-ink-faint); }
button.entry.minor::part(base) { color: var(--front-ink-faint); }
button.entry.danger { --button-ink: var(--system-danger-ink); --button-hover-ink: var(--system-danger-ink); }
button.entry.danger::part(base), button.entry.danger::part(base):hover, button.entry.danger:focus-visible::part(base) { color: var(--system-danger-ink); }
.side { flex: 1; min-inline-size: 0; position: relative; display: flex; align-items: center; justify-content: center; padding: var(--space-xxl); }
@media (width < 600px), (height < 600px) {
  [part="pause.shade"] { background: var(--system-pause-shade-compact); }
  [part="pause.frame"] { inline-size: 100%; padding: var(--space-xxl) var(--space-lg) var(--space-lg); }
  [part="pause.title"] { font-size: var(--system-title-size-compact); }
  [part="pause.status"] { inline-size: auto; align-self: stretch; margin-block-start: var(--space-md); padding: var(--space-sm) var(--space-md); }
  [part="pause.objective"] { font-size: calc(16 * var(--font-step-px)); }
  #Menu { margin-inline: calc(var(--space-lg) * -1); }
  button.entry { --button-font-size: var(--system-entry-size-compact); }
  .label { font-size: var(--system-entry-size-compact); }
  button.entry::part(base) { padding-inline: var(--space-xl) var(--space-lg); }
  button.entry.minor { --button-font-size: calc(16 * var(--font-step-px)); }
  #Divider { margin-inline-start: var(--space-xl); }
  .side { display: none; }
}
/* A short landscape phone: no status, the menu in two columns. */
@media (height < 600px) and (orientation: landscape) {
  [part="pause.frame"] { padding: var(--space-md) var(--space-lg); }
  [part="pause.status"] { display: none; }
  #Menu { display: grid; grid-template-columns: 1fr 1fr; column-gap: var(--space-md); padding-block-start: var(--space-sm); }
  #Divider { display: none; }
}
</style>
