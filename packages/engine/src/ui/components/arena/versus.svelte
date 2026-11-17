<meta name="component" content="ArenaVersus">
<meta name="strings" content="./arena.en.ogt">
<meta name="parts" content="ArenaVersus.frame ArenaVersus.side ArenaVersus.heading ArenaVersus.card ArenaVersus.portrait ArenaVersus.name ArenaVersus.rank ArenaVersus.badge ArenaVersus.plate ArenaVersus.picture ArenaVersus.label ArenaVersus.caption ArenaVersus.headline">
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
<section id="ArenaVersus" part="ArenaVersus.frame" class="bare" data-layout="row" aria-label={t("arena.versus.name")}>
  {#key at}
    <div class="stage" data-layout="row" data-gap="wide">
      <div part="ArenaVersus.side ArenaVersus.plate" class="side a" data-layout="stack" data-fill data-motion="enter" data-team="a" role="group" aria-label={nameA != '' ? nameA : t("arena.team.a")}>
        <h2 part="ArenaVersus.heading" class="team-name" data-team="a">{nameA != '' ? nameA : t("arena.team.a")}</h2>
        {#if sideA.length}
          <div class="cards" data-layout="stack" role="list">
            {#each sideA as f (f.id)}
              <div part="ArenaVersus.card" class="card" data-layout="row" data-gap="loose" role="listitem" aria-current={f.local} data-local={f.local}>
                <div part="ArenaVersus.portrait ArenaVersus.picture" class="portrait" data-layout="ratio">{#if f.portrait}<img class="art" src={f.portrait} alt="">{:else}<og-icon class="glyph" icon={f.icon}></og-icon>{/if}</div>
                <div class="who" data-layout="stack" data-gap="hair" data-fill>
                  <span part="ArenaVersus.name ArenaVersus.headline" class="name">{f.name}</span>
                  {#if f.character != ''}<span class="character" part="ArenaVersus.caption">{f.character}</span>{/if}
                  {#if f.rank != ''}<span part="ArenaVersus.rank ArenaVersus.label ArenaVersus.caption" class="rank">{f.rank}</span>{/if}
                  {#if f.local}<span class="you" part="ArenaVersus.badge">{t("arena.you")}</span>{/if}
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="waiting" part="ArenaVersus.caption" role="status">{t("arena.versus.waiting")}</p>
        {/if}
      </div>
      <div class="seal" data-layout="ratio" data-motion="stamp"><span part="ArenaVersus.badge" class="vs">{t("arena.versus.vs")}</span></div>
      <div part="ArenaVersus.side ArenaVersus.plate" class="side b" data-layout="stack" data-fill data-motion="enter" data-team="b" role="group" aria-label={nameB != '' ? nameB : t("arena.team.b")}>
        <h2 part="ArenaVersus.heading" class="team-name" data-team="b">{nameB != '' ? nameB : t("arena.team.b")}</h2>
        {#if sideB.length}
          <div class="cards" data-layout="stack" role="list">
            {#each sideB as f (f.id)}
              <div part="ArenaVersus.card" class="card" data-layout="row" data-gap="loose" role="listitem" aria-current={f.local} data-local={f.local}>
                <div part="ArenaVersus.portrait ArenaVersus.picture" class="portrait" data-layout="ratio">{#if f.portrait}<img class="art" src={f.portrait} alt="">{:else}<og-icon class="glyph" icon={f.icon}></og-icon>{/if}</div>
                <div class="who" data-layout="stack" data-gap="hair" data-fill>
                  <span part="ArenaVersus.name ArenaVersus.headline" class="name">{f.name}</span>
                  {#if f.character != ''}<span class="character" part="ArenaVersus.caption">{f.character}</span>{/if}
                  {#if f.rank != ''}<span part="ArenaVersus.rank ArenaVersus.label ArenaVersus.caption" class="rank">{f.rank}</span>{/if}
                  {#if f.local}<span class="you" part="ArenaVersus.badge">{t("arena.you")}</span>{/if}
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <p class="waiting" part="ArenaVersus.caption" role="status">{t("arena.versus.waiting")}</p>
        {/if}
      </div>
    </div>
  {/key}
</section>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); }
:host { inline-size: stretch; min-inline-size: 0; max-block-size: stretch; align-items: stretch; }
.stage { width: stretch; min-width: 0; }
[part="ArenaVersus.side ArenaVersus.plate"] { min-width: 0; padding: var(--space-md); }
[part="ArenaVersus.heading"] { margin: 0; text-align: center; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
[part="ArenaVersus.card"] { padding: var(--space-sm); }
.b [part="ArenaVersus.card"] { flex-direction: row-reverse; }
[part="ArenaVersus.portrait ArenaVersus.picture"] { --layout-ratio: 1; flex: none; width: var(--size-xl); height: var(--size-xl); overflow: hidden; }
.art { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.glyph { width: var(--size-md); height: var(--size-md); }
.who { min-width: 0; }
.name { --headline-text: var(--text-lg); }
.name, .character, .rank { max-inline-size: stretch; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; text-align: start; }
.rank { inline-size: stretch; white-space: normal; overflow-wrap: anywhere; }
.b .name, .b .character, .b .rank { text-align: end; }
.you { align-self: flex-start; white-space: nowrap; padding: 0 var(--space-xs); }
.b .you { align-self: flex-end; }
.waiting { margin: 0; padding: var(--space-lg); text-align: center; }
.seal { --layout-ratio: 1; flex: none; width: var(--size-xl); height: var(--size-xl); }
.vs { --badge-text: var(--text-xl2); white-space: nowrap; }
/* The reveal fills the screen, so the screen's width decides: a phone stacks the sides with the seal between them. */
@media (width < 600px) {
  .stage { flex-direction: column; align-items: stretch; --layout-gap-wide: var(--space-xs); }
  [part="ArenaVersus.side ArenaVersus.plate"] { flex: none; padding: var(--space-xs); --layout-gap: var(--space-xs); }
  .seal { align-self: center; width: var(--size-lg); height: var(--size-lg); }
  [part="ArenaVersus.portrait ArenaVersus.picture"] { width: var(--size-lg); height: var(--size-lg); }
  .glyph { width: var(--size-sm); height: var(--size-sm); }
}
</style>
