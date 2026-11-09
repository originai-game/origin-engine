<meta name="screen" content="credits" data-layer="menu" data-focus="" data-under="hide" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./credits.en.ogt">
<meta name="prop:sections" content="list" data-doc="The sections: { title, names: [{ name, role }] }.">
<meta name="prop:offset" content="number" data-doc="How far the roll has run, CSS px.">
<meta name="action:confirm" content="Faster">
<meta name="action:back" content="Back">
<meta name="fixture:default" content="{&quot;offset&quot;:0,&quot;sections&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;names&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Ada Lin&quot;,&quot;role&quot;:&quot;&quot;}],&quot;title&quot;:&quot;Directed by&quot;},{&quot;id&quot;:&quot;item-1&quot;,&quot;names&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Tomas Reyes&quot;,&quot;role&quot;:&quot;Lead artist&quot;},{&quot;id&quot;:&quot;item-1&quot;,&quot;name&quot;:&quot;Mira Okafor&quot;,&quot;role&quot;:&quot;Environment&quot;}],&quot;title&quot;:&quot;Art&quot;},{&quot;id&quot;:&quot;item-2&quot;,&quot;names&quot;:[{&quot;id&quot;:&quot;item-0&quot;,&quot;name&quot;:&quot;Jun Sato&quot;,&quot;role&quot;:&quot;Composer&quot;}],&quot;title&quot;:&quot;Music&quot;}]}">

<stack id="Credits" interactive="true" onclick={faster} onconfirm={faster}>
  <column id="Roll">
    <art id="Logo" kind="logo"></art>
    {#each $sections as s (s.id)}
      <column id="Section">
        <text id="Title">{s.title}</text>
        {#each s.names as n (n.id)}
          <column id="Person">
            <text id="Name">{n.name}</text>
            <text id="Role" hidden={!((n.role != ""))}>{n.role}</text>
          </column>
        {/each}
      </column>
    {/each}
    <column id="End">{@render end?.()}</column>
    <column id="Licences">{@render licences?.()}</column>
  </column>
  <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
</stack>

<style>
#Credits { width: stretch; height: stretch; background: var(--backdrop); }
#Roll { position: absolute; inset-inline-start: 0; inset-inline-end: 0; top: 0; padding: 48px; gap: 40px; align-items: center; translate: 0 bind("-offset"); }
@media (width < 600px), (height < 600px) {
#Roll { padding: 24px; gap: 28px; }
}
#Logo { width: 360px; height: 140px; }
@media (width < 600px), (height < 600px) {
#Logo { width: 240px; height: 94px; }
}
#Section { width: stretch; max-width: 640px; gap: 10px; align-items: center; }
#Title { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); text-align: center; }
#Person { align-items: center; gap: 2px; }
#Name { font: var(--text-xl); letter-spacing: var(--type-xl-tracking); text-align: center; }
#Role { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#End { width: stretch; max-width: 640px; }
#Licences { width: stretch; max-width: 640px; }
#Back { position: absolute; inset-inline-start: 16px; top: 16px; }
</style>
