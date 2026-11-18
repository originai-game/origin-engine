<meta name="screen" content="credits" data-layer="menu" data-focus="" data-under="hide" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./credits.en.ogt">
<meta name="parts" content="credits.frame credits.heading credits.name credits.shade credits.rule credits.role">
<meta name="prop:sections" content="list" data-doc="The sections: { title, names: [{ name, role }] }.">
<meta name="prop:offset" content="number" data-doc="How far the roll has run, CSS px.">
<meta name="prop:keyArt" content="number" data-doc="The key art's texture under the roll's veil (og.load.texture; 0: the style's dark ground).">
<meta name="prop:logo" content="number" data-doc="The game's logo texture at the head of the roll (og.load.texture; 0: the logo slot's placeholder).">
<meta name="action:confirm" content="Faster">
<meta name="action:back" content="Back">
<meta name="fixture:default" content="{&quot;keyArt&quot;:0,&quot;logo&quot;:0,&quot;offset&quot;:0,&quot;sections&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;names&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Ada Lin&quot;,&quot;role&quot;:&quot;&quot;}],&quot;title&quot;:&quot;Directed by&quot;},{&quot;id&quot;:&quot;item-1&quot;,&quot;names&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Tomas Reyes&quot;,&quot;role&quot;:&quot;Lead artist&quot;},{&quot;id&quot;:&quot;item-1&quot;,&quot;name&quot;:&quot;Mira Okafor&quot;,&quot;role&quot;:&quot;Environment&quot;}],&quot;title&quot;:&quot;Art&quot;},{&quot;id&quot;:&quot;item-2&quot;,&quot;names&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Jun Sato&quot;,&quot;role&quot;:&quot;Composer&quot;}],&quot;title&quot;:&quot;Music&quot;}]}">

<div id="Credits" part="credits.frame" class="bare" data-layout="shell" data-gap="none" interactive="true" onclick={faster} onconfirm={faster}>
  {#if keyArt != 0}<art id="Key" kind="keyArt" src={keyArt}></art>{/if}
  <div id="Veil" part="credits.shade"></div>
  <header id="Bar" data-layout="row"><button id="Back" variant="quiet" onclick={back}><div class="back"><og-icon class="chevron" icon="chevronLeft"></og-icon><span>{t("screen.common.back")}</span></div></button></header>
  <div id="Window" data-fill="true">
  <div id="Roll">
    <art id="Logo" kind="logo" src={logo}></art>
    {#each sections as s (s.id)}
      <section id="Section">
        <div id="Head"><span class="rule" part="credits.rule"></span><h2 id="Title" part="credits.heading">{s.title}</h2><span class="rule end" part="credits.rule"></span></div>
        {#each s.names as n (n.id)}
          <div id="Person">
            <p id="Name" part="credits.name">{n.name}</p>
            {#if n.role != ""}<p id="Role" part="credits.role">{n.role}</p>{/if}
          </div>
        {/each}
      </section>
    {/each}
    <div id="End">{@render end?.()}</div>
    <div id="Licences">{@render licences?.()}</div>
  </div>
  </div>
  <div id="Faster"><span class="key">{t("screen.credits.fasterKey")}</span><span>{t("screen.credits.faster")}</span></div>
</div>

<style>
/* Back has a bar of its own above the roll's window, so the roll never
   runs under it. The roll is one centred column the game scrolls up by
   its offset, cut to its window, which fades at both edges; under it the
   key art behind a veil darkest where the names run. */
:root { position: relative; block-size: stretch; overflow: hidden; background: var(--front-ground); color: var(--front-ink); }
#Key, [part="credits.shade"] { position: absolute; inset: 0; }
#Key { inline-size: 100%; block-size: 100%; object-fit: cover; }
[part="credits.shade"] { background: var(--front-roll-veil); }
#Bar { position: relative; padding: var(--space-lg) var(--space-xxl); }
.back { display: flex; align-items: center; column-gap: var(--space-xs); }
.back .chevron { inline-size: var(--size-xs); block-size: var(--size-xs); }
.chevron:dir(rtl) { scale: -1 1; }
#Window { position: relative; overflow: hidden; mask-image: linear-gradient(180deg, transparent, #000 10%, #000 85%, transparent); }
#Roll { position: absolute; inset-inline: 0; inset-block-start: 0; display: flex; flex-direction: column; align-items: center; row-gap: var(--space-3xl); padding: var(--space-3xl) var(--space-lg); translate: 0 bind("-offset"); }
/* The logo is as wide as the roll's column up to a measure, so it (and a
   missing logo's caption) fits a phone at any text size. */
#Logo { inline-size: 100%; max-inline-size: calc(460 * var(--font-step-px)); aspect-ratio: 2.4; block-size: auto; object-fit: contain; filter: var(--front-logo-lift); }
#Section, #End, #Licences { inline-size: 100%; max-inline-size: var(--measure-xl); }
#Section, #Person { display: flex; flex-direction: column; align-items: center; text-align: center; }
#Section { row-gap: var(--space-lg); }
#Person { row-gap: var(--space-xxs); }
h2, p { margin: 0; }
#Head { display: flex; align-items: center; justify-content: center; column-gap: var(--space-md); inline-size: 100%; }
[part="credits.rule"] { flex: 0 1 calc(40 * var(--font-step-px)); min-inline-size: var(--space-sm); block-size: var(--stroke-hairline); background: var(--front-rule); }
[part="credits.rule"].end { scale: -1 1; }
[part="credits.heading"] { flex: 0 1 auto; min-inline-size: 0; font-size: calc(13 * var(--font-step-px)); font-weight: var(--font-weight-bold); letter-spacing: var(--front-heading-tracking); text-transform: uppercase; color: var(--front-accent); overflow-wrap: anywhere; }
[part="credits.name"] { font-size: calc(30 * var(--font-step-px)); line-height: var(--font-leading-tight); font-weight: var(--font-weight-semibold); text-shadow: var(--front-shadow); overflow-wrap: anywhere; }
[part="credits.role"] { font-size: calc(13 * var(--font-step-px)); font-weight: var(--font-weight-semibold); letter-spacing: var(--font-tracking-xl); text-transform: uppercase; color: var(--front-ink-faint); overflow-wrap: anywhere; }
#Faster { position: absolute; inset-inline-end: var(--space-xl); inset-block-end: var(--space-lg); display: flex; align-items: center; column-gap: var(--space-sm); font-size: calc(13 * var(--font-step-px)); color: var(--front-ink-dim); }
.key { padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.6; font-weight: var(--font-weight-bold); color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); }
@media (pointer: coarse) { #Faster { display: none; } }
@media (width < 600px), (height < 600px) {
  [part="credits.shade"] { background: var(--front-veil); }
  #Bar { padding: var(--space-sm) var(--space-md); }
  #Roll { padding: var(--space-xl) var(--space-lg); row-gap: var(--space-xxl); }
  [part="credits.heading"] { letter-spacing: var(--hud-label-tracking); }
  [part="credits.name"] { font-size: calc(22 * var(--font-step-px)); }
  #Faster { display: none; }
}
</style>
