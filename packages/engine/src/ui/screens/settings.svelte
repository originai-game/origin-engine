<meta name="screen" content="settings" data-layer="menu" data-focus="Tree" data-under="dim" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="back">
<meta name="strings" content="./settings.en.ogt">
<meta name="prop:note" content="string" data-doc="A line under the title (Some changes apply after a restart); empty: none.">
<meta name="action:confirm" content="Select">
<meta name="action:back" content="Back">
<meta name="action:tabNext" content="Next page">
<meta name="action:tabPrev" content="Previous page">
<meta name="fixture:default" content="{&quot;note&quot;:&quot;&quot;}">

<column id="Screen">
  <row id="Header">
    <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
    <column id="Heading">
      <text id="Title" heading="true">{t("screen.settings.title")}</text>
      <text id="Note" hidden={!(($note != ""))}>{$note}</text>
    </column>
  </row>
  <column id="Tree">{@render tree?.()}</column>
</column>

<style>
#Screen { width: stretch; height: stretch; padding: 32px; gap: 16px; }
@media phonePortrait phoneLandscape {
#Screen { padding: 16px; gap: 12px; }
}
#Header { width: stretch; gap: 12px; align-items: center; }
#Heading { width: stretch; gap: 2px; }
#Title { font: var(--text-xl2); }
@media phonePortrait phoneLandscape {
#Title { font: var(--text-xl); }
}
#Note { font: var(--text-caption); color: var(--ink-muted); line-clamp: 2; }
#Tree { width: stretch; height: stretch; }
</style>
