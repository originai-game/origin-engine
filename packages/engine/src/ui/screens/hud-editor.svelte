<meta name="screen" content="hud-editor" data-layer="modal" data-focus="" data-under="none" data-input="menu" data-pause="true" data-capture="true" data-remember="true" data-back="back">
<meta name="strings" content="./hud-editor.en.ogt">
<meta name="parts" content="hud-editor.hint hud-editor.dock">
<meta name="prop:showHint" content="bool" data-doc="The hint at the top shows (false once the player has moved something).">
<meta name="action:back" content="Done">
<meta name="fixture:default" content="{&quot;showHint&quot;:true}">

<stack id="RootEditor">
  <panel id="Hint" hidden={!($showHint)}>
    <text id="Text" part="hud-editor.hint">{t("screen.hud-editor.hint")}</text>
  </panel>
  <column id="EditorEditor" part="hud-editor.dock">{@render editorEditor?.()}</column>
</stack>

<style>
:root { --surface-saturation: 120%; }
@media (prefers-color-scheme: light) { :root { --surface-saturation: 200%; } }
#RootEditor { width: stretch; height: stretch; }
#Hint { position: absolute; top: var(--space-xl); inset-inline: 0; margin-inline: auto; width: max-content; max-width: 100%; padding: var(--space-md); background: none; border: none; box-shadow: none; border-radius: var(--radius-button); backdrop-filter: blur(var(--glass-blur-regular)) saturate(var(--surface-saturation)); -og-material: glass regular float; }
panel[id=Hint]::part(frame) { border: none; }
@media (width < 600px), (height < 600px) {
#Hint { top: var(--space-3xl); }
}
#Text { text-align: center; line-clamp: 2; }
#EditorEditor { position: absolute; bottom: var(--space-xl); inset-inline: 0; margin-inline: auto; width: max-content; max-width: 100%; }
@media (width < 600px) and (orientation: portrait) {
#EditorEditor { inset-inline-start: var(--space-lg); inset-inline-end: 0; bottom: var(--space-xxl); margin-inline: 0; width: auto; }
}
@media (height < 600px) and (orientation: landscape) {
#EditorEditor { inset-inline-start: var(--size-xxl); inset-inline-end: 0; bottom: var(--space-xl); margin-inline: 0; width: auto; }
}
</style>
