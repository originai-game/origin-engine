<meta name="screen" content="hud-editor" data-layer="modal" data-focus="" data-under="none" data-input="menu" data-pause="true" data-capture="true" data-remember="true" data-back="back">
<meta name="strings" content="./hud-editor.en.ogt">
<meta name="prop:showHint" content="bool" data-doc="The hint at the top shows (false once the player has moved something).">
<meta name="action:back" content="Done">
<meta name="fixture:default" content="{&quot;showHint&quot;:true}">

<stack id="RootEditor">
  <panel id="Hint" hidden={!($showHint)}>
    <text id="Text">{t("screen.hud-editor.hint")}</text>
  </panel>
  <column id="EditorEditor">{@render editorEditor?.()}</column>
</stack>

<style>
:root { --surface-saturation: 120%; }
@media (prefers-color-scheme: light) { :root { --surface-saturation: 200%; } }
#RootEditor { width: stretch; height: stretch; }
#Hint { position: absolute; top: 24px; -og-center-x: true; padding: 10px; background: none; border: none; box-shadow: none; border-radius: var(--radius-button); backdrop-filter: blur(22px) saturate(var(--surface-saturation)); -og-material: glass regular float; }
panel[id=Hint]::part(frame) { border: none; }
@media (width < 600px), (height < 600px) {
#Hint { top: 56px; }
}
#Text { text-align: center; line-clamp: 2; }
#EditorEditor { position: absolute; bottom: 24px; -og-center-x: true; }
@media (width < 600px) and (orientation: portrait) {
#EditorEditor { inset-inline-start: 8px; inset-inline-end: 8px; bottom: 40px; }
}
@media (height < 600px) and (orientation: landscape) {
#EditorEditor { inset-inline-start: 52px; inset-inline-end: 52px; bottom: 24px; }
}
</style>
