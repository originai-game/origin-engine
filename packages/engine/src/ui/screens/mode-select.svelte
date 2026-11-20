<meta name="screen" content="mode-select" data-layer="menu" data-focus="Modes" data-under="dim" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./mode-select.en.ogt">
<meta name="prop:modes" content="list" data-doc="The modes: { name, line, tag, locked, reason, texture }.">
<meta name="prop:selected" content="string" data-doc="The stable collection key selected now.">
<meta name="prop:keyArt" content="number" data-doc="The key art's texture under the screen's veil (og.load.texture; 0: the style's dark ground).">
<meta name="action:confirm" content="Play">
<meta name="action:back" content="Back">
<meta name="fixture:default" content="{&quot;keyArt&quot;:0,&quot;modes&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;line&quot;:&quot;The campaign, alone or with a friend.&quot;,&quot;locked&quot;:false,&quot;name&quot;:&quot;Story&quot;,&quot;reason&quot;:&quot;&quot;,&quot;tag&quot;:&quot;&quot;,&quot;texture&quot;:0},{&quot;id&quot;:&quot;item-1&quot;,&quot;line&quot;:&quot;Ranked fights, one life.&quot;,&quot;locked&quot;:false,&quot;name&quot;:&quot;Arena&quot;,&quot;reason&quot;:&quot;&quot;,&quot;tag&quot;:&quot;Ranked&quot;,&quot;texture&quot;:0},{&quot;id&quot;:&quot;item-2&quot;,&quot;line&quot;:&quot;Hard challenges for the brave.&quot;,&quot;locked&quot;:true,&quot;name&quot;:&quot;Trials&quot;,&quot;reason&quot;:&quot;Finish chapter 2&quot;,&quot;tag&quot;:&quot;New&quot;,&quot;texture&quot;:0}],&quot;selected&quot;:&quot;item-0&quot;}">

<div id="Screen" part="frame" variant="bare">
  {#if keyArt != 0}<art id="Key" kind="keyArt" src={keyArt}></art>{/if}
  <div class="shade" id="Veil" part="shade"></div>
  <header class="band" id="Header" part="band">
    <button id="Back" variant="quiet" onclick={back}><div class="back"><og-icon class="chevron" icon="chevronLeft"></og-icon><span>{t("screen.common.back")}</span></div></button>
    <h1 class="title" id="Title" part="title">{t("screen.mode-select.title")}</h1>
    <hr id="Rule">
  </header>
  <div id="Scroll">
    <div id="Modes" focus-group="wrap">
      {#each modes as mode (mode.id)}
        <article class="cell" id="Mode" part="cell" interactive="true" focusable="true" onclick={select} class:selection-off={mode.id != selected} class:selection-on={mode.id == selected} class:lock-open={!mode.locked} class:lock-locked={mode.locked}>
          <div class="plate" id="Plate" part="plate">
          <art id="Art" kind="levelThumb" src={mode.texture}></art>
          <div class="foot" id="Foot" part="foot"></div>
          {#if mode.tag}<span class="tag" id="Tag" part="tag" class:fresh={mode.locked}>{mode.tag}</span>{/if}
          <div id="Body">
            <h2 class="name" id="Name" part="name">{mode.name}</h2>
            <p id="Line">{mode.line}</p>
            {#if mode.locked}<div id="Reason" part="lock"><og-icon class="lock" icon="lock"></og-icon><span>{mode.reason}</span></div>{/if}
          </div>
          <div class="plate-after"></div></div>
        </article>
      {/each}
    </div>
  </div>
  <div id="Detail">{@render detail?.()}</div>
  <footer id="Footer">
    <button id="Play" class="go" variant="primary" disabled={modes.length == 0} onclick={play} onconfirm={play}>{t("screen.mode-select.play")}</button>
  </footer>
</div>

<style>
/* The key art under a veil; the band (Back, the title, a rule) at the
   top; the modes as full-art cards with two cut corners, three across, two
   on a tablet, one column that scrolls on a phone; Play at the foot. */
#Screen { position: relative; display: flex; flex-direction: column; row-gap: var(--space-xl); block-size: stretch; overflow: hidden; padding: var(--space-xxl) var(--space-3xl) var(--space-xl); background: var(--front-ground); color: var(--front-ink); }
#Key, .shade { position: absolute; inset: 0; }
#Key { inline-size: 100%; block-size: 100%; object-fit: cover; }
.shade { background: var(--front-veil); }
h1, h2, p { margin: 0; }
.band, #Scroll, #Detail, #Footer { position: relative; }
.band { display: flex; align-items: center; column-gap: var(--space-lg); }
.back { display: flex; align-items: center; column-gap: var(--space-xs); }
.back .chevron { inline-size: var(--size-xs); block-size: var(--size-xs); }
.chevron:dir(rtl) { scale: -1 1; }
.title { flex: 0 1 auto; min-inline-size: 0; font-size: var(--front-title-size); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); font-stretch: var(--front-stretch); letter-spacing: var(--font-tracking-xs); text-shadow: var(--front-shadow); }
#Rule { flex: 1 1 0; min-inline-size: 0; block-size: var(--stroke-hairline); margin: 0; border: 0; background: var(--front-divider); }
/* The column reaches into the screen's padding by the chosen card's glow,
   so the glow is not cut at its edges. */
#Scroll { flex: 1; min-block-size: 0; overflow: hidden auto; margin: calc(var(--space-lg) * -1); padding: var(--space-lg); }
#Modes { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); grid-auto-rows: minmax(calc(320 * var(--font-step-px)), 1fr); gap: var(--space-xl); block-size: 100%; }
/* A card holds its plate: the art fills the plate, a dark foot holds its
   words, two corners are cut and the rim is drawn over the art. The card
   itself is not cut, so its glow and its focus ring fall outside. */
.cell { position: relative; display: flex; border: 0; background: none; border-radius: 0; box-shadow: none; }
.plate { position: relative; flex: 1 1 auto; display: flex; flex-direction: column; justify-content: end; overflow: hidden; border-radius: var(--front-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.plate .plate-after { position: absolute; inset: 0; box-shadow: var(--front-card-rim); }
.cell:hover, .cell:focus-visible { translate: 0 var(--state-lift); }
.cell:active { translate: 0 0; }
#Art, .foot { position: absolute; inset: 0; }
#Art { inline-size: 100%; block-size: 100%; object-fit: cover; }
.foot { background: var(--front-card-foot); }
/* The chosen card ringed and lit in the accent; the others sit back; a
   locked one goes grey and dark, its words kept. */
.selection-on { filter: var(--front-card-on-glow); }
.selection-on .plate .plate-after { box-shadow: var(--front-card-on-rim); }
.selection-off #Art { filter: var(--front-art-off); }
.lock-locked #Art { filter: var(--front-art-locked); }
#Body { position: relative; display: flex; flex-direction: column; row-gap: var(--space-xs); padding: var(--space-xl); }
.name { font-size: var(--front-name-size); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); font-stretch: var(--front-stretch); letter-spacing: var(--font-tracking-xs); text-shadow: var(--front-shadow); overflow-wrap: anywhere; }
#Line { font-size: calc(17 * var(--font-step-px)); line-height: var(--font-leading-snug); color: var(--front-ink-dim); line-clamp: 2; }
#Reason { display: flex; align-items: center; column-gap: var(--space-xs); font-size: calc(14 * var(--font-step-px)); font-weight: var(--font-weight-semibold); color: var(--front-lock-ink); }
.lock { flex: none; inline-size: var(--size-xs); block-size: var(--size-xs); }
.tag { position: absolute; inset-block-start: var(--space-lg); inset-inline-end: var(--space-lg); max-inline-size: calc(100% - 2 * var(--space-lg)); padding: var(--space-xxs) var(--space-sm); font-size: var(--hud-tag-size); font-weight: var(--font-weight-bold); letter-spacing: var(--hud-label-tracking); text-transform: uppercase; color: var(--front-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.tag.fresh { color: var(--front-tag-on-ink); background: var(--front-tag-on-fill); }
#Footer { display: flex; justify-content: end; }
/* Play: the screen's one lit plate, cut at two corners like the cards,
   glowing while there is a mode to play. */
#Play { --button-font-size: calc(20 * var(--font-step-px)); --button-tracking: var(--font-tracking-sm); --button-transform: uppercase;  }
#Play, #Play { min-inline-size: calc(240 * var(--font-step-px)); block-size: var(--control-lg); }
/* Only the plate is cut; the focus ring stays whole around it. */
@media (600px <= width < 1024px) and (height >= 600px) {
  #Screen { padding-inline: var(--space-xxl); }
  #Modes { grid-template-columns: repeat(2, minmax(0, 1fr)); block-size: auto; }
}
#Screen:size(compact) { row-gap: var(--space-md); padding: var(--space-lg) var(--space-lg) var(--space-md); }
#Rule:size(compact) { display: none; }
#Body:size(compact) { padding: var(--space-md); row-gap: var(--space-xxs); }
.name:size(compact) { font-size: var(--front-name-size-compact); }
#Line:size(compact) { font-size: calc(14 * var(--font-step-px)); }
.tag:size(compact) { inset-block-start: var(--space-sm); inset-inline-end: var(--space-sm); }
#Play:size(compact) { flex: 1; }
#Play:size(compact), #Play:size(compact) { min-inline-size: 0; }
/* A phone held upright: the title takes its own row under Back, the cards
   one column that scrolls and fades at its foot. */
.band:size(narrow) { flex-wrap: wrap; row-gap: var(--space-xs); }
#Back:size(narrow) { margin-inline-start: calc(var(--space-sm) * -1); }
.title:size(narrow) { flex: 1 0 100%; font-size: var(--front-title-size-compact); }
#Modes:size(narrow) { grid-template-columns: minmax(0, 1fr); grid-auto-rows: minmax(calc(170 * var(--font-step-px)), auto); gap: var(--space-md); block-size: auto; }
#Scroll:size(narrow) { mask-image: linear-gradient(180deg, #000 calc(100% - var(--space-xl)), transparent); padding-block-end: var(--space-xl); }
</style>
