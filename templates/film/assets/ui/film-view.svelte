<meta name="screen" content="film-view" data-layer="hud">
<meta name="parts" content="film-view.picture">
<meta name="prop:picture" content="number" data-doc="The texture og.film.view gives for the segment playing (0 before the first frame).">
<meta name="fixture:default" content="{&quot;picture&quot;:0}">

<!-- The footage fills the surface, so film-choices' hotspot fractions land on the picture (film §4). -->
<div id="FilmView" class="bare">
  <img id="Picture" part="film-view.picture" src={picture} alt="">
</div>

<style>
#FilmView { position: fixed; inset: 0; background: var(--film-shade); }
[part="film-view.picture"] { position: absolute; inset: 0; inline-size: 100%; block-size: 100%; object-fit: cover; }
</style>
