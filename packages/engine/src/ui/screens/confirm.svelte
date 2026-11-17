<meta name="component" content="confirm">
<meta name="parts" content="confirm.title confirm.body confirm.actions">
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

<dialog id="Confirm" modal data-layout="shell" closedby="closerequest" bind:open={open} aria-label={title}>
  <h2 id="Title" part="confirm.title">{title}</h2>
  <div id="Scroll" data-layout="scroll" data-fill hidden={body == ""}>
    <p id="Body" part="confirm.body">{body}</p>
  </div>
  <footer id="Buttons" part="confirm.actions" data-layout="cluster">
    <button id="Cancel" variant="quiet" autofocus onclick={cancel}>{cancel}</button>
    <button id="Ok" variant={danger ? "danger" : "primary"} onclick={confirm}>{confirm}</button>
  </footer>
</dialog>

<style>
/* The question and the actions stay put; what follows from it scrolls
   between them when the dialog is taller than the screen. The dialog is as
   tall as what it holds, up to the screen's safe height. */
#Confirm { inline-size: var(--modal-width); block-size: auto; gap: var(--space-md); }
#Title, #Buttons { flex: none; }
/* What follows keeps a line of its own however long the question is: the
   body type's line, which grows with the player's text scale. */
#Scroll { flex: 0 1 auto; min-block-size: var(--type-body-line-height); }
#Body { margin: 0; color: var(--ink-muted); }
/* The actions share the row and fill it. Where both do not fit on one line
   each takes a line of its own, the confirming one above the way out. */
#Buttons { flex-wrap: wrap-reverse; gap: var(--space-sm); margin-top: var(--space-sm); }
/* An action longer than the dialog is wide wraps inside its button. */
#Buttons button { flex: 1 1 auto; max-inline-size: 100%; white-space: normal; }
/* A wrapped action keeps clear of its button's frame above and below. */
#Buttons button::part(base) { padding-block: var(--button-wrap-pad, var(--space-sm)); }
/* A phone: the question is the style's title one step down, so a long one
   leaves room for what follows. */
@media (width < 600px), (height < 500px) {
  #Confirm { --title-size: var(--title-lead-size); }
}
/* A phone on its side has width to spare and little height: the dialog
   widens with what it holds, so a long question takes fewer lines and the
   actions share one. */
@media (height < 500px) and (orientation: landscape) {
  #Confirm { inline-size: fit-content; min-inline-size: var(--modal-width); max-inline-size: var(--measure-xxl); }
}
</style>
