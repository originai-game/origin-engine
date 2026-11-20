<meta name="screen" content="hud-editor" data-layer="modal" data-focus="" data-under="none" data-input="menu" data-pause="true" data-capture="true" data-remember="true" data-back="back">
<meta name="strings" content="./hud-editor.en.ogt">
<meta name="parts" content="hud-editor.hint hud-editor.dock">
<meta name="prop:showHint" content="bool" data-doc="The hint at the top shows (false once the player has moved something).">
<meta name="action:back" content="Done">
<meta name="fixture:default" content="{&quot;showHint&quot;:true}">

<div id="RootEditor">
  {#if showHint}
    <div id="Hint">
      <p id="Text" part="hud-editor.hint">{t("screen.hud-editor.hint")}</p>
    </div>
  {/if}
  <div id="EditorEditor" part="hud-editor.dock">{@render editorEditor?.()}</div>
</div>

<style>
:root { --surface-saturation: 120%; position: relative; block-size: stretch; }
@media (prefers-color-scheme: light) { :root { --surface-saturation: 200%; } }
p { margin: 0; }
/* The hint over the top on a cut glass plate, like the pause menu's
   status, and the game's editor dock at the foot, each centred. */
#Hint, [part="hud-editor.dock"] { position: absolute; inset-inline: 0; margin-inline: auto; inline-size: max-content; max-inline-size: 100%; }
#Hint { inset-block-start: var(--space-xl); max-inline-size: calc(100% - 2 * var(--space-lg)); padding: var(--space-md) var(--space-xl); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); backdrop-filter: blur(var(--glass-blur-regular)) saturate(var(--surface-saturation)); border-radius: var(--front-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
[part="hud-editor.hint"] { text-align: center; line-clamp: 2; font-size: calc(15 * var(--font-step-px)); font-weight: var(--font-weight-semibold); color: var(--front-ink); }
[part="hud-editor.dock"] { inset-block-end: var(--space-xl); }
@media (width < 600px), (height < 600px) {
  #Hint { inset-block-start: var(--space-3xl); }
}
@media (width < 600px) and (orientation: portrait) {
  [part="hud-editor.dock"] { inset-inline: var(--space-lg) 0; inset-block-end: var(--space-xxl); margin-inline: 0; inline-size: auto; }
}
@media (height < 600px) and (orientation: landscape) {
  [part="hud-editor.dock"] { inset-inline: var(--size-xxl) 0; margin-inline: 0; inline-size: auto; }
}
</style>
