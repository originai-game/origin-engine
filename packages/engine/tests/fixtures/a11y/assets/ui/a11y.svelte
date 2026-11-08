<meta name="component" content="a11y">
<meta name="prop:settings" content="bool" data-doc="The settings dialog is open (written back false when the player dismisses it).">
<meta name="prop:pause" content="bool" data-doc="The pause dialog is open (written back false when the player dismisses it).">
<meta name="prop:tab" content="string" data-doc="The settings section shown: audio or graphics.">
<meta name="prop:master" content="number" data-doc="The master volume, 0 to 1.">
<meta name="prop:music" content="number" data-doc="The music volume, 0 to 1.">
<meta name="prop:muted" content="bool" data-doc="Mute everything.">
<meta name="prop:subtitles" content="bool" data-doc="Subtitles on.">
<meta name="prop:contrast" content="bool" data-doc="High contrast.">

<column id="Screens">
<dialog id="Settings" modal closedby="any" bind:open={$settings} aria-label="Settings">
  <h2 id="Title">Settings</h2>
  <nav id="Section" role="radiogroup" aria-label="Section">
    <label class="tab"><input type="radio" id="Audio" value="audio" bind:group={$tab}>Audio</label>
    <label class="tab"><input type="radio" id="Graphics" value="graphics" bind:group={$tab}>Graphics</label>
  </nav>
  {#if $tab == "audio"}
    <div class="volume">
      <p class="name">Master</p>
      <input type="range" id="Master" aria-label="Master" min="0" max="1" step="0.05" bind:value={$master}>
    </div>
    <div class="volume">
      <p class="name">Music</p>
      <input type="range" id="Music" aria-label="Music" min="0" max="1" step="0.05" bind:value={$music}>
    </div>
    <label class="row">Mute everything<input type="checkbox" switch id="Mute" bind:checked={$muted}></label>
    <label class="row">Subtitles<input type="checkbox" switch id="Subtitles" bind:checked={$subtitles}></label>
  {:else}
    <label class="row">High contrast<input type="checkbox" switch id="Contrast" bind:checked={$contrast}></label>
  {/if}
  <footer id="Footer">
    <button id="Done" variant="primary" onclick={done}>Done</button>
  </footer>
</dialog>

<dialog id="Pause" modal closedby="any" bind:open={$pause} aria-label="Paused">
  <h2 id="PauseTitle">Paused</h2>
  <button id="Resume" variant="primary" autofocus onclick={resume}>Resume</button>
</dialog>
</column>

<style>
#Settings { width: 600px; gap: 12px; }
#Title, #PauseTitle { margin: 0; }
#Section { display: flex; gap: 8px; width: stretch; }
.volume { display: flex; align-items: center; gap: 16px; width: stretch; }
.volume .name { width: 140px; margin: 0; }
.row { width: stretch; }
#Footer { display: flex; justify-content: end; width: stretch; }
#Pause { gap: 12px; }
</style>
