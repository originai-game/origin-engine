<meta name="screen" content="pause" data-layer="menu" data-focus="Panel/Menu/Resume" data-under="blur" data-input="menu" data-pause="true" data-capture="true" data-remember="true" data-back="resume">
<meta name="strings" content="./pause.en.ogt">
<meta name="parts" content="pause.frame pause.title pause.objective">
<meta name="prop:objective" content="string" data-doc="The current objective (empty: none).">
<meta name="prop:played" content="number" data-doc="Seconds played this run.">
<meta name="prop:canSave" content="bool" data-doc="Saving is allowed here.">
<meta name="action:confirm" content="Select">
<meta name="action:back" content="Resume">
<meta name="fixture:default" content="{&quot;canSave&quot;:true,&quot;objective&quot;:&quot;Reach the lighthouse&quot;,&quot;played&quot;:4520}">

<div id="Pause">
  <section id="Panel" part="pause.frame">
    <h2 id="Title" part="pause.title">{t("screen.pause.title")}</h2>
    <div id="Status">
      {#if objective != ""}<p id="Objective" part="pause.objective">{objective}</p>{/if}
      <p id="Played">{t("screen.pause.Played", { played: formatDuration(played) })}</p>
    </div>
    <div id="Menu" focus-group="wrap">
      <button id="Resume" variant="primary" onclick={resume}>{t("screen.pause.resume")}</button>
      <button id="Settings" onclick={settings}>{t("screen.pause.settings")}</button>
      <button id="Controls" onclick={controls}>{t("screen.pause.controls")}</button>
      <button id="Save" disabled={!canSave} onclick={save}>{t("screen.pause.save")}</button>
      <button id="Load" onclick={load}>{t("screen.pause.load")}</button>
      <button id="QuitToMenu" variant="quiet" onclick={quitToMenu}>{t("screen.pause.quitToMenu")}</button>
      <button id="Quit" variant="danger" onclick={quit}>{t("screen.pause.quit")}</button>
    </div>
  </section>
  <aside id="Side" class="side">{@render side?.()}</aside>
</div>

<style>
/* The panel centred in the screen, the game's side content beside it. */
:root { display: flex; gap: var(--space-xl); align-items: center; justify-content: center; block-size: stretch; padding: var(--space-xxl); }
[part="pause.frame"] { display: flex; flex-direction: column; gap: var(--space-lg); flex: 1; max-inline-size: var(--measure-lg); transition: opacity var(--motion-enter), translate var(--motion-enter); }
@starting-style { [part="pause.frame"] { opacity: 0; translate: 0 var(--space-xl); } }
[part="pause.title"] { margin: 0; font-size: var(--type-xl2-size); letter-spacing: var(--type-xl2-tracking); }
p { margin: 0; }
[part="pause.objective"] { color: var(--ink-muted); line-clamp: 2; }
#Status { display: flex; flex-direction: column; gap: var(--space-xs); }
#Played { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#Menu { display: grid; gap: var(--space-sm); }
@media (width < 600px), (height < 600px) {
  :root { padding: var(--space-lg); }
  .side { display: none; }
}
/* A short landscape phone: the menu in two columns, no status. */
@media (height < 600px) and (orientation: landscape) {
  [part="pause.frame"] { max-inline-size: var(--measure-xl); gap: var(--space-sm); }
  #Status { display: none; }
  #Menu { grid-template-columns: 1fr 1fr; }
}
</style>
