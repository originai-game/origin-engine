<meta name="component" content="ArenaVersus">
<meta name="strings" content="./arena.en.ogt">
<meta name="prop:nameA" content="string" data-doc="The first side's team name; empty takes the kit's (Blue team).">
<meta name="prop:nameB" content="string" data-doc="The second side's team name; empty takes the kit's (Red team).">
<meta name="prop:sideA" content="list" data-doc="The first side's fighters in the game's order, keyed rows of { id, name, portrait, icon, character, rank, local }: portrait a texture (0 shows the icon); character the champion's name (empty hides); rank as the game words it (Gold II, 1840 SR; empty hides); local marks the local player (You).">
<meta name="prop:sideB" content="list" data-doc="The second side's fighters, keyed as sideA. An empty side reads Waiting for opponents.">
<meta name="prop:at" content="number" data-doc="The reveal's id: a new value replays the sides' entrance.">
<meta name="fixture:default" content='{"nameA":"","nameB":"","at":1,"sideA":[{"id":"you","name":"Rhea","portrait":0,"icon":"sword","character":"Vex","rank":"Gold II","local":true}],"sideB":[{"id":"rook","name":"Rook","portrait":0,"icon":"shield","character":"Brenna","rank":"Gold I","local":false}]}'>
<meta name="fixture:teams" content='{"nameA":"Iron Wolves","nameB":"Ember Court","at":2,"sideA":[{"id":"kai","name":"Kai","portrait":0,"icon":"shield","character":"Brenna","rank":"Platinum III","local":false},{"id":"you","name":"Rhea","portrait":0,"icon":"sword","character":"Vex","rank":"Gold II","local":true},{"id":"ana","name":"Ana","portrait":0,"icon":"heart","character":"Sela","rank":"Gold IV","local":false}],"sideB":[{"id":"rook","name":"Rook","portrait":0,"icon":"bolt","character":"Idris","rank":"Gold I","local":false},{"id":"wren","name":"Wren","portrait":0,"icon":"leaf","character":"Maren","rank":"","local":false},{"id":"quill","name":"Quill","portrait":0,"icon":"star","character":"Quill","rank":"Silver I","local":false}]}'>
<meta name="fixture:waiting" content='{"nameA":"","nameB":"","at":3,"sideA":[{"id":"you","name":"Rhea","portrait":0,"icon":"sword","character":"Vex","rank":"Gold II","local":true}],"sideB":[]}'>
<meta name="fixture:overflow" content='{"nameA":"The Unbroken Wardens of the Ninefold Gate","nameB":"Ember Court Invitational Reserve","at":4,"sideA":[{"id":"a1","name":"Maximilian Vandenberghe-Okonkwo","portrait":0,"icon":"crown","character":"Ysolde the Unbroken, Warden of the Ninefold Gate","rank":"Grandmaster · 3,912 SR · Top 0.1%","local":false},{"id":"a2","name":"Rhea","portrait":0,"icon":"sword","character":"Vex","rank":"Diamond I","local":true},{"id":"a3","name":"Ana","portrait":0,"icon":"heart","character":"Sela","rank":"Diamond II","local":false},{"id":"a4","name":"Kai","portrait":0,"icon":"shield","character":"Brenna","rank":"Diamond III","local":false},{"id":"a5","name":"Jo","portrait":0,"icon":"music","character":"Tamsin","rank":"Master","local":false}],"sideB":[{"id":"b1","name":"Abernathy-Whitcombe","portrait":0,"icon":"bolt","character":"Idris","rank":"Grandmaster","local":false},{"id":"b2","name":"Wren","portrait":0,"icon":"leaf","character":"Maren","rank":"Master","local":false},{"id":"b3","name":"Rook","portrait":0,"icon":"flag","character":"Kestrel","rank":"Diamond I","local":false},{"id":"b4","name":"Moreau","portrait":0,"icon":"fire","character":"Ashen Pyre","rank":"Diamond IV","local":false},{"id":"b5","name":"Laurent","portrait":0,"icon":"gem","character":"Hollow","rank":"","local":false}]}'>

<!--
  The match reveal: the two sides facing each other across a VS seal, each
  fighter a card with portrait (or icon), name, champion and rank as the
  game words it; the local player is marked You. A side the game has not
  filled reads Waiting for opponents. A new reveal id (at) replays the
  entrance, which reduced motion skips. Read only.
-->
<section id="ArenaVersus" part="frame" variant="bare" layout="row" aria-label={t("arena.versus.name")}>
  {#key at}
    <div class="stage" layout="row" gap="wide">
      <div part="plate" class="side a" layout="stack" fill play="enter" team="a" role="group" aria-label={nameA != '' ? nameA : t("arena.team.a")}>
        <h2 part="heading" class="team-name heading" team="a">{nameA != '' ? nameA : t("arena.team.a")}</h2>
        {#if sideA.length}
          <div class="cards" layout="stack" role="list">
            {#each sideA as f (f.id)}
              <div part="card" class="card" layout="row" gap="loose" role="listitem" aria-current={f.local} class:local={f.local}>
                <div part="picture" class="portrait ratio">{#if f.portrait}<img class="art" src={f.portrait} alt="">{:else}<og-icon class="glyph" icon={f.icon}></og-icon>{/if}</div>
                <div class="who" layout="stack" gap="hair" fill>
                  <span part="headline" class="name">{f.name}</span>
                  {#if f.character}<span class="character" part="caption">{f.character}</span>{/if}
                  {#if f.rank}<span part="caption" class="rank label">{f.rank}</span>{/if}
                  {#if f.local}<span class="you" part="badge">{t("arena.you")}</span>{/if}
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="waiting" part="caption" role="status">{t("arena.versus.waiting")}</p>
        {/if}
      </div>
      <div class="seal ratio" play="stamp"><div class="seal-edge ratio"><div class="seal-face ratio"><span part="badge" class="vs">{t("arena.versus.vs")}</span></div></div></div>
      <div part="plate" class="side b" layout="stack" fill play="enter" team="b" role="group" aria-label={nameB != '' ? nameB : t("arena.team.b")} class:team-b={true}>
        <h2 part="heading" class="team-name heading" team="b" class:team-b={true}>{nameB != '' ? nameB : t("arena.team.b")}</h2>
        {#if sideB.length}
          <div class="cards" layout="stack" role="list">
            {#each sideB as f (f.id)}
              <div part="card" class="card" layout="row" gap="loose" role="listitem" aria-current={f.local} class:local={f.local}>
                <div part="picture" class="portrait ratio">{#if f.portrait}<img class="art" src={f.portrait} alt="">{:else}<og-icon class="glyph" icon={f.icon}></og-icon>{/if}</div>
                <div class="who" layout="stack" gap="hair" fill>
                  <span part="headline" class="name">{f.name}</span>
                  {#if f.character}<span class="character" part="caption">{f.character}</span>{/if}
                  {#if f.rank}<span part="caption" class="rank label">{f.rank}</span>{/if}
                  {#if f.local}<span class="you" part="badge">{t("arena.you")}</span>{/if}
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="waiting" part="caption" role="status">{t("arena.versus.waiting")}</p>
        {/if}
      </div>
    </div>
  {/key}
</section>

<style>
.ratio { display: flex; align-items: center; justify-content: center; aspect-ratio: var(--layout-ratio); overflow: hidden; }
.ratio img { inline-size: 100%; block-size: 100%; object-fit: cover; }
/* The versus reveal over the scene: each side's name large in its colour
   over a rule of it, its players on rows washed in that colour from the
   outer edge (the second side mirrored), the local player's row lit
   white; between them a gold hexagon seal. */
#ArenaVersus { inline-size: stretch; min-inline-size: 0; max-block-size: stretch; align-items: stretch; color: var(--hud-ink); }
.stage { width: stretch; min-width: 0; }
.side { min-width: 0; container-type: inline-size; padding: 0; border: 0; border-radius: 0; background: none; box-shadow: none; }
/* A long team name breaks only between words; one wider than its side
   (a wide face) steps its type down until it fits three lines. */
.heading { margin: 0; padding-block-end: var(--space-xs); text-align: start; font-size: var(--party-size-side); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--result-team-a-ink); filter: var(--result-team-a-glow); box-shadow: var(--result-team-a-rule); text-shadow: none; overflow-wrap: normal; line-clamp: 3; -og-fit-min: base; }
.heading.team-b { text-align: end; color: var(--result-team-b-ink); filter: var(--result-team-b-glow); box-shadow: var(--result-team-b-rule); }
.card { padding: var(--space-xs) var(--space-md); border: 0; border-radius: 0; color: var(--hud-ink); background: var(--result-team-a-row); box-shadow: var(--result-team-a-edge); text-shadow: none; }
.b .card { flex-direction: row-reverse; background: var(--party-team-b-row-end); box-shadow: var(--party-team-b-edge-end); }
.card.local { background: var(--result-team-a-you-row); color: var(--chosen-ink); }
.b .card.local { background: var(--party-team-b-you-row); }
.portrait { --layout-ratio: 1; flex: none; box-sizing: border-box; padding: var(--stroke-thick); border: 0; border-radius: 0; background: var(--hud-edge); box-shadow: none; clip-path: var(--hud-octagon); overflow: hidden; inline-size: var(--size-xl); block-size: var(--size-xl); background: var(--hud-edge); }
.art { width: stretch; height: stretch; object-fit: cover; object-position: center top; clip-path: var(--hud-octagon); }
.glyph { width: var(--size-md); height: var(--size-md); color: var(--hud-ink); }
.local .glyph { color: var(--chosen-ink); }
.who { min-width: 0; }
.name { max-inline-size: stretch; text-align: start; font-size: var(--party-size-fighter); line-height: 1.15; font-weight: var(--hud-figure-weight); color: inherit; overflow-wrap: break-word; }
.character { max-inline-size: stretch; text-align: start; font-size: var(--hud-size-xs); line-height: 1.3; font-weight: var(--hud-label-weight); color: var(--hud-ink-dim); overflow-wrap: break-word; }
.rank { inline-size: stretch; text-align: start; font-size: var(--hud-size-xs); line-height: 1.3; font-weight: var(--hud-label-weight); letter-spacing: 0; text-transform: none; color: var(--hud-gold); overflow-wrap: break-word; }
.local .character, .local .rank { color: var(--party-lit-dim); }
.b .name, .b .character, .b .rank { text-align: end; }
.you { align-self: flex-start; flex: none; padding: 0 var(--space-xs); border: 0; font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); text-shadow: none; border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; white-space: nowrap; }
.b .you { align-self: flex-end; }
.waiting { margin: 0; padding: var(--space-lg); text-align: center; font-size: var(--hud-size-sm); color: var(--hud-ink-dim); text-shadow: var(--hud-shadow); }
.seal { --layout-ratio: 0.87; flex: none; inline-size: calc(var(--party-size-seal) * 2); filter: var(--party-seal-glow); overflow: visible; }
.seal-edge { --layout-ratio: 0.87; box-sizing: border-box; inline-size: stretch; padding: var(--stroke-thick); clip-path: var(--hud-hexagon); background: var(--hud-gold); }
.seal-face { --layout-ratio: 0.87; inline-size: stretch; clip-path: var(--hud-hexagon); background: var(--party-seal-fill); }
.vs { padding: 0; border-radius: 0; white-space: nowrap; font-size: var(--party-size-seal); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-style: var(--hud-figure-slant); color: var(--hud-gold); background: none; box-shadow: none; text-shadow: none; }
/* A narrow side (the reveal in a column of a wide page at large text):
   the portrait smaller and the row tighter, so a long name keeps whole. */
@container (width < 300px) {
  .card { padding: var(--space-xxs) var(--space-xs); --layout-gap-loose: var(--layout-gap-tight); }
  .portrait { inline-size: var(--party-portrait-compact); block-size: var(--party-portrait-compact); }
  .glyph { width: var(--size-sm); height: var(--size-sm); }
}
/* The reveal fills the screen, so the screen's width decides: a phone stacks the sides with the seal between them. */
@media (width < 600px) {
  .stage { flex-direction: column; align-items: stretch; --layout-gap-wide: var(--space-xs); }
  .side { flex: none; --layout-gap: var(--space-xs); }
  .heading { font-size: var(--party-size-hero-compact); }
  .seal { align-self: center; inline-size: calc(var(--party-size-seal-compact) * 2); }
  .vs { font-size: var(--party-size-seal-compact); }
  .card { padding: var(--space-xxs) var(--space-sm); --layout-gap-loose: var(--layout-gap); }
  .portrait { inline-size: var(--party-portrait-compact); block-size: var(--party-portrait-compact); }
  .glyph { width: var(--size-sm); height: var(--size-sm); }
}
</style>
