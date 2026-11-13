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

<dialog id="Confirm" modal closedby="closerequest" bind:open={$open} aria-label={$title}>
  <h2 id="Title" part="confirm.title">{$title}</h2>
  <p id="Body" part="confirm.body" hidden={$body == ""}>{$body}</p>
  <footer id="Buttons" part="confirm.actions">
    <button id="Cancel" variant="quiet" autofocus onclick={cancel}>{$cancel}</button>
    <button id="Ok" variant={$danger ? "danger" : "primary"} onclick={confirm}>{$confirm}</button>
  </footer>
</dialog>

<style>
#Confirm { width: var(--modal-width); gap: var(--space-md); }
#Body { margin: 0; color: var(--ink-muted); }
#Buttons { display: flex; justify-content: end; gap: var(--space-sm); margin-top: var(--space-sm); }
</style>
