<meta name="component" content="RpgQuestDetail">
<meta name="strings" content="./rpg.en.ogt">
<meta name="prop:name" content="string" data-doc="The quest's name, in the heading band.">
<meta name="prop:summary" content="string" data-doc="Its story so far, as the game writes it.">
<meta name="prop:objectives" content="list" data-doc="Keyed rows of { id, text, done, optional, step }: step 0 is a stage, 1 a task under it; done ticks it, optional marks it so.">
<meta name="prop:rewards" content="list" data-doc="Keyed rows of { id, icon, label, detail }: detail a second, quieter line (Unique relic); empty hides it.">
<meta name="prop:label" content="string" data-doc="What kind of quest it is, as the game words it (Main quest), over the name; empty hides it.">
<meta name="prop:tracked" content="bool" data-doc="The HUD follows this quest: its action reads Stop tracking.">
<meta name="prop:canTrack" content="bool" data-doc="Show the Track (or Stop tracking) action, which reports track.">
<meta name="prop:canLocate" content="bool" data-doc="Show the Show on map action, which reports locate.">
<meta name="prop:scenery" content="number" data-doc="A texture beside the rewards (og.load.texture), with its own space; 0 is none.">
<meta name="fixture:default" content='{"name":"The Hollow Crown","summary":"The crown is said to lie within the ruins.","objectives":[{"id":"o1","text":"Enter the ruins","done":false,"optional":false,"step":0},{"id":"o2","text":"Find the crown","done":true,"optional":false,"step":1}],"rewards":[{"id":"g","icon":"coin","label":"1,200 Gold","detail":""}],"scenery":0,"label":"Main quest","tracked":false,"canTrack":true,"canLocate":true}'>
<meta name="fixture:long" content='{"name":"The Hollow Crown of Caer Velin","label":"Main quest","summary":"The crown is said to lie within the ruins of Caer Velin, where the old kings were buried with their oaths. The abbot’s map marks a way in beneath the eastern tower, but the gravekeeper warns that the oath-bound do not sleep, and that whoever wears the crown inherits every promise its kings broke.","objectives":[{"id":"o1","text":"Reach the highland ruins","done":true,"optional":false,"step":0},{"id":"o2","text":"Enter the ruins of Caer Velin","done":false,"optional":false,"step":0},{"id":"o3","text":"Find the abbot’s map","done":true,"optional":false,"step":1},{"id":"o4","text":"Light the three rune pillars (1 / 3)","done":false,"optional":false,"step":1},{"id":"o5","text":"Speak with the gravekeeper before nightfall at the chapel gate","done":false,"optional":true,"step":1},{"id":"o6","text":"Descend into the oath-crypt","done":false,"optional":false,"step":0},{"id":"o7","text":"Find the crown","done":false,"optional":false,"step":0}],"rewards":[{"id":"g","icon":"coin","label":"1,200 Gold","detail":""},{"id":"x","icon":"star","label":"3,400 Experience","detail":""},{"id":"c","icon":"crown","label":"Crown of Caer Velin","detail":"Unique relic"}],"scenery":0,"tracked":true,"canTrack":true,"canLocate":true}'>

<!--
  One quest: its kind in gold small capitals over its name on the heading
  band (over the scenery, shaded into the glass, when the game gives one),
  the story so far, its objectives as stages with their tasks on a rail
  under them, and its rewards as cut chips, each with its icon in an
  octagon socket (one with a detail, a unique reward, lit gold). A done
  objective is struck with its diamond filled in the success colour; the
  first stage not done is lit gold. The body scrolls inside the frame's
  height, fading out above the actions (Show on map, Track or Stop tracking), which stay at its
  foot; they report locate and track. Track is the primary action; once
  the quest is tracked it reads Stop tracking on a quiet secondary button. With neither action it is read only.
-->
<article id="RpgQuestDetail" part="frame" layout="stack" fill aria-label={name} class:with-scenery={scenery != 0}>
  <header part="band" class="hero" layout="overlay">
    {#if scenery}<img part="picture" class="scenery" anchor="fill" src={scenery} alt=""><div class="shade" anchor="fill"></div>{/if}
    <div class="heading" layout="stack" gap="hair" anchor="bottom-start">
      {#if label}<span class="kind">{label}</span>{/if}
      <h2 part="title" class="name">{name}</h2>
    </div>
  </header>
  <div class="body-wrap" layout="overlay" fill>
  <div class="body" layout="scroll" gap="loose">
    {#if summary}<p class="summary">{summary}</p>{/if}
    {#if objectives.length}<section class="part" layout="stack" gap="tight" aria-label={t("rpg.quest.objectives")}>
      <div class="part-head" part="headline" layout="row" gap="tight" role="heading"><span>{t("rpg.quest.objectives")}</span><span class="rule" fill></span></div>
      <ul class="objectives" layout="stack" gap="none">
        {#each objectives as o, i (o.id)}
          <li part="row" class="objective" layout="row" gap="loose" data-optional={o.optional} class:step-1={(o.step) == '1'} state:current={o.step == 0 && !o.done && (i == 0 || objectives[i - 1]['done'])} state:done={o.done}>
            <div class="objective-mark" part="dot"></div>
            <span class="objective-text" fill>{o.text}</span>
            {#if o.optional}<span class="objective-optional" part="caption">{t("rpg.quest.optional")}</span>{/if}
          </li>
        {/each}
      </ul>
    </section>{/if}
    {#if rewards.length}<section class="part" part="well" layout="stack" gap="tight" aria-label={t("rpg.quest.rewards")}>
      <div class="part-head" part="headline" layout="row" gap="tight" role="heading"><span>{t("rpg.quest.rewards")}</span><span class="rule" fill></span></div>
      <ul class="rewards" layout="wrap" gap="tight">
        {#each rewards as r (r.id)}
          <li part="figure" class="reward" layout="row" gap="loose" class:unique={!!r.detail}>
            <div class="socket" layout="overlay" class:unique={!!r.detail}><div class="socket-face cover" anchor="fill" layout="stack"><og-icon class="reward-mark" icon={r.icon} data-principal></og-icon></div></div>
            <div class="reward-text" layout="stack" gap="none" fill><span class="reward-label">{r.label}</span>{#if r.detail}<span class="reward-detail" part="caption">{r.detail}</span>{/if}</div>
          </li>
        {/each}
      </ul>
    </section>{/if}
  </div>
  {#if canTrack || canLocate}<div part="fade" class="fade" anchor="bottom-edge"></div>{/if}
  </div>
  {#if canTrack || canLocate}<footer part="actions" class="actions" layout="wrap" gap="tight">
    {#if canLocate}<button id="Locate" onclick={locate}><div class="face" layout="row" gap="hair"><og-icon class="action-mark" icon="map"></og-icon><span>{t("rpg.quest.locate")}</span></div></button>{/if}
    {#if canTrack && tracked}<button id="Untrack" onclick={track}>{t("rpg.quest.untrack")}</button>{:else if canTrack}<button id="Track" class="go" variant="primary" onclick={track}>{t("rpg.quest.track")}</button>{/if}
  </footer>{/if}
</article>

<style>
.cover { justify-content: center; align-items: center; }
/* The band carries the title over the scenery, so it wears neither fill nor rule of its own here. */
.hero { width: stretch; flex: none; min-height: var(--size-xxl); padding: 0; border-bottom: 0; }
.with-scenery .hero { min-height: calc(var(--size-xxl) * 2); }
.scenery { width: stretch; height: stretch; object-fit: cover; object-position: center 38%; }
.shade { background: var(--quest-banner-shade); }
.heading { padding: var(--space-md) var(--space-lg) var(--space-sm); min-width: 0; max-width: stretch; }
.kind { font: var(--caption-text); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--quest-gold); }
.name { min-inline-size: 0; overflow-wrap: anywhere; font-size: var(--quest-size-title); text-shadow: var(--quest-banner-shadow); }
.body-wrap { width: stretch; min-height: 0; }
.body { width: stretch; height: stretch; padding: var(--space-xs) var(--space-lg) var(--space-lg); }
/* The body fades into the frame above the actions, so a cut line reads as more to scroll. */
.fade { width: stretch; }
.summary { margin: 0; max-inline-size: var(--measure-lg); color: var(--quest-ink-quiet); }
.part { flex: none; width: stretch; }
.part-head { margin: 0; font: var(--caption-text); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--quest-ink-quiet); }
.rule { height: var(--stroke-hairline); background: var(--quest-rail); }
.objectives, .rewards { margin: 0; padding: 0; list-style: none; width: stretch; }
.objective { padding: var(--space-xxs) 0; min-width: 0; font-weight: var(--quest-weight); }
/* A task hangs from its stage on a rail. */
.objective.step-1 { margin-inline-start: var(--space-xs); padding-inline-start: var(--space-lg); font-weight: var(--headline-weight); border-inline-start: var(--quest-rail-edge); color: var(--quest-ink-quiet); }
.objective-mark { flex: none; width: var(--space-sm); height: var(--space-sm); margin-inline: var(--space-xxs); rotate: 45deg; border-radius: 0; --dot-fill: transparent; box-shadow: var(--quest-step-rim); }
.objective.step-1 .objective-mark { width: var(--space-xs); height: var(--space-xs); }
.objective:state(current) .objective-mark { --dot-fill: var(--quest-gold); box-shadow: var(--quest-step-lit); }
.objective:state(done) { color: var(--quest-ink-faint); text-decoration-line: var(--hud-done-line); }
.objective:state(done) .objective-mark { --dot-fill: var(--tone-success); box-shadow: var(--quest-step-done); }
.objective-text { min-inline-size: 0; overflow-wrap: anywhere; }
.objective-optional { flex: none; padding: 0 var(--space-xs); white-space: nowrap; letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); background: var(--quest-chip-fill); box-shadow: var(--quest-chip-rim); border-radius: var(--quest-stamp-radius); }
/* A reward: a cut chip, its icon in an octagon socket; a unique one lit gold. */
.reward { min-inline-size: calc(var(--size-xxl) * 2.6); max-inline-size: stretch; flex: 1 1 auto; padding: var(--space-xs) var(--space-md) var(--space-xs) var(--space-xs); background: var(--quest-entry-fill); box-shadow: var(--quest-entry-rim); border-radius: var(--quest-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.reward.unique { box-shadow: var(--quest-ready-rim); }
.socket { flex: none; width: var(--size-lg); height: var(--size-lg); background: var(--quest-socket-rim); clip-path: polygon(22% 0, 78% 0, 100% 22%, 100% 78%, 78% 100%, 22% 100%, 0 78%, 0 22%); --layout-anchor-inset: var(--stroke-thin); }
.socket.unique { background: var(--quest-socket-gold-rim); }
.socket-face { background: var(--quest-socket-fill); clip-path: polygon(22% 0, 78% 0, 100% 22%, 100% 78%, 78% 100%, 22% 100%, 0 78%, 0 22%); color: var(--ink-bright); }
.socket.unique .socket-face { background: var(--quest-socket-gold-fill); color: var(--quest-gold); }
.reward-mark { width: var(--size-sm); height: var(--size-sm); }
.reward-text { min-width: 0; overflow-wrap: anywhere; }
.reward-label { font-weight: var(--quest-weight); color: var(--ink-bright); }
.reward.unique .reward-detail { color: var(--quest-gold); }
/* The actions stay at the frame's foot while the body scrolls. */
.actions { width: stretch; flex: none; justify-content: flex-end; padding: var(--space-sm) var(--space-lg) var(--space-md); }
.actions button { min-height: var(--size-touch); }
.action-mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
.heading:size(compact), .body:size(compact) { padding-inline: var(--space-md); }
.name:size(compact) { font-size: var(--title-size); }
.reward:size(compact) { min-inline-size: 0; width: stretch; }
.actions:size(compact) { padding-inline: var(--space-md); }
.actions button:size(compact) { flex: 1 1 auto; }
</style>
