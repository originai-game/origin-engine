<meta name="screen" content="pause" data-layer="menu" data-focus="Panel/Menu/Resume" data-under="blur" data-input="menu" data-pause="true" data-capture="true" data-remember="true" data-back="resume">
<meta name="strings" content="./pause.en.ogt">
<meta name="parts" content="pause.frame pause.title pause.objective">
<meta name="prop:objective" content="string" data-doc="The current objective (empty: none).">
<meta name="prop:played" content="number" data-doc="Seconds played this run.">
<meta name="prop:canSave" content="bool" data-doc="Saving is allowed here.">
<meta name="action:confirm" content="Select">
<meta name="action:back" content="Resume">
<meta name="fixture:default" content="{&quot;canSave&quot;:true,&quot;objective&quot;:&quot;Reach the lighthouse&quot;,&quot;played&quot;:4520}">

<row id="Pause">
  <column id="Panel" part="pause.frame">
    <text id="Title" part="pause.title" heading="true">{t("screen.pause.title")}</text>
    <column id="Status">
      <text id="Objective" part="pause.objective" hidden={!(($objective != ""))}>{$objective}</text>
      <text id="Played">{t("screen.pause.Played", { played: formatDuration($played) })}</text>
    </column>
    <grid id="Menu" focus-group="wrap">
      <button id="Resume" variant="primary" onclick={resume}>{t("screen.pause.resume")}</button>
      <button id="Settings" onclick={settings}>{t("screen.pause.settings")}</button>
      <button id="Controls" onclick={controls}>{t("screen.pause.controls")}</button>
      <button id="Save" disabled={!($canSave)} onclick={save}>{t("screen.pause.save")}</button>
      <button id="Load" onclick={load}>{t("screen.pause.load")}</button>
      <button id="QuitToMenu" variant="quiet" onclick={quitToMenu}>{t("screen.pause.quitToMenu")}</button>
      <button id="Quit" variant="danger" onclick={quit}>{t("screen.pause.quit")}</button>
    </grid>
  </column>
  <column id="Side">{@render side?.()}</column>
</row>

<style>
#Pause { width: stretch; height: stretch; padding: var(--space-xxl); gap: var(--space-xl); justify-content: center; align-items: center; }
@media (width < 600px), (height < 600px) {
#Pause { padding: var(--space-lg); }
}
#Panel { width: stretch; max-width: var(--measure-lg); gap: var(--space-lg); opacity: 1; translate: 0 0; transition: opacity var(--motion-enter), translate var(--motion-enter); }
@starting-style { #Panel { opacity: 0; translate: 0 var(--space-xl); } }
@media (height < 600px) and (orientation: landscape) {
#Panel { width: stretch; max-width: var(--measure-xl); gap: var(--space-sm); }
}
#Title { font-size: var(--type-xl2-size); }
#Status { width: stretch; gap: var(--space-xs); }
@media (height < 600px) and (orientation: landscape) {
#Status { display: none; }
}
#Objective { color: var(--ink-muted); line-clamp: 2; }
#Played { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#Menu { width: stretch; gap: var(--space-sm); grid-template-columns: 1fr; }
@media (height < 600px) and (orientation: landscape) {
#Menu { grid-template-columns: 1fr 1fr; }
}
#Resume { width: stretch; }
#Settings { width: stretch; }
#Controls { width: stretch; }
#Save { width: stretch; }
#Load { width: stretch; }
#QuitToMenu { width: stretch; }
#Quit { width: stretch; }
@media (width < 600px), (height < 600px) {
#Side { display: none; }
}
</style>
