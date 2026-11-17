<meta name="screen" content="credits" data-layer="menu" data-focus="" data-under="hide" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./credits.en.ogt">
<meta name="parts" content="credits.frame credits.heading credits.name">
<meta name="prop:sections" content="list" data-doc="The sections: { title, names: [{ name, role }] }.">
<meta name="prop:offset" content="number" data-doc="How far the roll has run, CSS px.">
<meta name="action:confirm" content="Faster">
<meta name="action:back" content="Back">
<meta name="fixture:default" content="{&quot;offset&quot;:0,&quot;sections&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;names&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Ada Lin&quot;,&quot;role&quot;:&quot;&quot;}],&quot;title&quot;:&quot;Directed by&quot;},{&quot;id&quot;:&quot;item-1&quot;,&quot;names&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Tomas Reyes&quot;,&quot;role&quot;:&quot;Lead artist&quot;},{&quot;id&quot;:&quot;item-1&quot;,&quot;name&quot;:&quot;Mira Okafor&quot;,&quot;role&quot;:&quot;Environment&quot;}],&quot;title&quot;:&quot;Art&quot;},{&quot;id&quot;:&quot;item-2&quot;,&quot;names&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Jun Sato&quot;,&quot;role&quot;:&quot;Composer&quot;}],&quot;title&quot;:&quot;Music&quot;}]}">

<div id="Credits" part="credits.frame" class="bare" data-layout="shell" data-gap="none" interactive="true" onclick={faster} onconfirm={faster}>
  <header id="Bar" data-layout="row"><button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button></header>
  <div id="Window" data-fill="true">
  <div id="Roll">
    <art id="Logo" kind="logo"></art>
    {#each sections as s (s.id)}
      <section id="Section">
        <h2 id="Title" part="credits.heading">{s.title}</h2>
        {#each s.names as n (n.id)}
          <div id="Person">
            <p id="Name" part="credits.name">{n.name}</p>
            {#if n.role != ""}<p id="Role">{n.role}</p>{/if}
          </div>
        {/each}
      </section>
    {/each}
    <div id="End">{@render end?.()}</div>
    <div id="Licences">{@render licences?.()}</div>
  </div>
  </div>
</div>

<style>
/* Back has a bar of its own above the roll's window, so the roll never
   runs under it. The roll is one centred column the game scrolls up by
   its offset, cut to its window. */
:root { block-size: stretch; background: var(--backdrop); }
#Bar { padding: var(--space-sm) var(--space-lg); }
#Window { position: relative; overflow: hidden; }
#Roll { position: absolute; inset-inline: 0; inset-block-start: 0; display: flex; flex-direction: column; align-items: center; gap: var(--space-3xl); padding: var(--space-3xl); translate: 0 bind("-offset"); }
/* The logo is as wide as the roll's column up to a measure, so it (and a
   missing logo's caption) fits a phone at any text size. */
#Logo { inline-size: 100%; max-inline-size: var(--measure-md); block-size: var(--size-3xl); }
#Section, #End, #Licences { inline-size: 100%; max-inline-size: var(--measure-xl); }
#Section, #Person { display: flex; flex-direction: column; align-items: center; text-align: center; }
#Section { gap: var(--space-md); }
#Person { gap: var(--space-xxs); }
h2, p { margin: 0; }
[part="credits.heading"] { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); }
[part="credits.name"] { font: var(--text-xl); letter-spacing: var(--type-xl-tracking); }
#Role { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
@media (width < 600px), (height < 600px) {
  #Roll { padding: var(--space-xl); gap: var(--space-xl); }
  #Logo { block-size: var(--size-xxl); }
}
</style>
