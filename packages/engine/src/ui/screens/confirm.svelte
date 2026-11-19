<meta name="component" content="confirm">
<meta name="strings" content="./confirm.en.ogt">
<meta name="parts" content="confirm.title confirm.body confirm.actions confirm.kicker">
<meta name="prop:open" content="bool" data-doc="Shown (written back false when the player dismisses it).">
<meta name="prop:title" content="string" data-doc="Its question, the dialog's heading and accessible name.">
<meta name="prop:body" content="string" data-doc="What follows from it; empty: none.">
<meta name="prop:confirm" content="string" data-doc="The confirming button's words (Quit, Delete).">
<meta name="prop:cancel" content="string" data-doc="The cancelling button's words.">
<meta name="prop:danger" content="bool" data-doc="Confirming destroys something: the confirm button is the danger variant.">
<meta name="action:confirm" content="Confirm">
<meta name="action:cancel" content="Cancel">
<meta name="fixture:default" content="{&quot;open&quot;:true,&quot;title&quot;:&quot;Quit to the menu?&quot;,&quot;body&quot;:&quot;Progress since the last save is lost.&quot;,&quot;confirm&quot;:&quot;Quit&quot;,&quot;cancel&quot;:&quot;Keep playing&quot;,&quot;danger&quot;:true}">
<meta name="fixture:overflow" content="{&quot;open&quot;:true,&quot;title&quot;:&quot;Delete the save of the Ninefold Gate campaign for good?&quot;,&quot;body&quot;:&quot;Chapter 7, 61 hours played, every companion and every item in the vault. This cannot be undone, and the cloud copy is deleted with it.&quot;,&quot;confirm&quot;:&quot;Delete everything&quot;,&quot;cancel&quot;:&quot;Keep the save&quot;,&quot;danger&quot;:true}">

<dialog id="Confirm" modal data-layout="shell" closedby="closerequest" bind:open={open} aria-label={title} class:danger={danger}>
  {#if danger}<div id="Kicker" part="confirm.kicker"><span class="mark"></span><span>{t("screen.confirm.undone")}</span></div>{/if}
  <h2 id="Title" part="confirm.title">{title}</h2>
  <div id="Scroll" data-layout="scroll" data-fill hidden={body == ""}>
    <p id="Body" part="confirm.body">{body}</p>
  </div>
  <footer id="Buttons" part="confirm.actions" data-layout="cluster">
    <button id="Cancel" class="plate glass" autofocus onclick={cancel}>{cancel}</button>
    <button id="Ok" class="plate go" variant={danger ? "danger" : "primary"} onclick={confirm}>{confirm}</button>
  </footer>
</dialog>

<style>
/* A dense smoked plate over a dark scrim, lit along its top in the accent,
   or in red under a kicker when confirming destroys something. The
   question and the actions stay put; what follows from it scrolls between
   them when the dialog is taller than the screen. The dialog is as tall
   as what it holds, up to the screen's safe height. */
#Confirm { --panel-fill: var(--dialog-fill); --panel-radius: 0; --panel-border-width: 0; --panel-shadow: var(--dialog-edge), var(--dialog-lift); --dialog-backdrop: var(--dialog-scrim); inline-size: calc(460 * var(--font-step-px)); max-inline-size: 100%; block-size: auto; gap: var(--space-sm); padding: var(--space-xl) var(--space-xl) var(--space-lg); color: var(--front-ink); }
#Confirm.danger { --panel-shadow: var(--dialog-danger-edge), var(--dialog-lift); }
#Kicker, #Title, #Buttons { flex: none; }
[part="confirm.kicker"] { display: flex; align-items: center; column-gap: var(--space-sm); font-size: var(--hud-label-size); font-weight: var(--font-weight-bold); letter-spacing: var(--hud-label-tracking); text-transform: uppercase; color: var(--system-danger-ink); }
[part="confirm.kicker"]:lang(zh, ja, ko) { font-size: calc(14 * var(--font-step-px)); letter-spacing: 0; }
.mark { flex: none; inline-size: calc(8 * var(--font-step-px)); block-size: calc(8 * var(--font-step-px)); rotate: 45deg; background: var(--system-danger-ink); }
[part="confirm.title"] { margin: 0; font-size: var(--dialog-title-size); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); font-stretch: var(--front-stretch); letter-spacing: var(--font-tracking-xs); color: var(--front-ink); text-shadow: var(--front-shadow); }
/* What follows keeps a line of its own however long the question is. */
#Scroll { flex: 0 1 auto; min-block-size: calc(var(--dialog-body-size) * var(--font-leading-normal)); }
#Body { margin: 0; font-size: var(--dialog-body-size); line-height: var(--font-leading-normal); color: var(--front-ink-dim); }
/* The actions share the row and fill it. Where both do not fit on one line
   each takes a line of its own, the confirming one above the way out. */
#Buttons { flex-wrap: wrap-reverse; gap: var(--space-sm); margin-top: var(--space-md); }
/* An action longer than the dialog is wide wraps inside its button. */
#Buttons button { flex: 1 1 auto; max-inline-size: 100%; min-block-size: var(--touch-target); white-space: normal; }
/* A wrapped action keeps clear of its button's frame above and below. */
#Buttons button::part(base) { padding-block: var(--button-wrap-pad, var(--space-sm)); }
button.plate { --button-radius: 0; --button-focus-radius: 0; --button-border-width: 0; --button-font-size: var(--dialog-label-size); --button-tracking: var(--font-tracking-lg); --button-transform: uppercase; }
button.plate::part(base) { border-width: 0; clip-path: polygon(var(--dialog-cut) 0, 100% 0, 100% calc(100% - var(--dialog-cut)), calc(100% - var(--dialog-cut)) 100%, 0 100%, 0 var(--dialog-cut)); }
/* Chinese, Japanese and Korean have no capitals to track: the label is set a step larger instead. */
button.plate:lang(zh, ja, ko) { --button-font-size: var(--dialog-label-cjk-size); --button-tracking: 0; }
button.glass { --button-fill: var(--hud-plate-fill); --button-hover-fill: var(--hud-plate-fill); --button-pressed-fill: var(--hud-plate-fill); --button-base-shadow: var(--system-plate-edge); --button-hover-shadow: var(--system-plate-lit-edge); --button-rim: transparent; --button-hover-rim: transparent; --button-ink: var(--front-ink); --button-hover-ink: var(--front-ink); }
#Ok { --button-danger-fill: var(--dialog-danger-fill); --button-danger-hover-fill: var(--dialog-danger-hover-fill); --button-danger-pressed-fill: var(--dialog-danger-pressed-fill); --button-danger-ink: var(--front-ink); --button-danger-rim: transparent; --button-danger-shadow: none; }
/* A phone: the question one step down, so a long one leaves room for what
   follows. */
@media (width < 600px), (height < 500px) {
  #Confirm { padding: var(--space-lg); }
  [part="confirm.title"] { font-size: var(--dialog-title-size-compact); }
}
/* A phone on its side has width to spare and little height: the dialog
   widens with what it holds, so a long question takes fewer lines and the
   actions share one. */
@media (height < 500px) and (orientation: landscape) {
  #Confirm { inline-size: fit-content; min-inline-size: var(--modal-width); max-inline-size: var(--measure-xxl); }
}
</style>
