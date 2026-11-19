<meta name="component" content="Profile">
<meta name="strings" content="./profile.en.ogt">
<meta name="parts" content="Profile.frame Profile.identity Profile.avatar Profile.name Profile.handle Profile.rank Profile.bio Profile.stats Profile.stat Profile.label Profile.value Profile.detail Profile.actions Profile.action Profile.reason Profile.state Profile.spinner Profile.retry Profile.title Profile.caption Profile.headline Profile.cell Profile.figure Profile.fade">
<meta name="prop:name" content="string" data-doc="Player display name; empty uses Player.">
<meta name="prop:handle" content="string" data-doc="Stable player handle or discriminator, as the game words it; empty hides it.">
<meta name="prop:portrait" content="number" data-doc="Loaded portrait texture; zero uses the base Avatar fallback.">
<meta name="prop:level" content="number" data-doc="Level shown by Avatar; zero hides it.">
<meta name="prop:status" content="string" data-doc="Avatar presence: online, away, busy or offline; empty hides it.">
<meta name="prop:rank" content="string" data-doc="Game-formatted rank or title; empty hides it.">
<meta name="prop:bio" content="string" data-doc="Player biography or activity; wraps, empty hides it.">
<meta name="prop:stats" content="list" data-doc="Keyed { id, label, value, detail } readouts. The game formats all values and decides their order; detail is an optional explanation.">
<meta name="prop:actions" content="list" data-doc="Keyed { id, label, disabled, reason, pending } actions, e.g. friend, invite, edit or report. action reports the stable row id; pending disables it and reads Sending. The game owns permissions and effects.">
<meta name="prop:privateStats" content="bool" data-doc="Hide statistics and show Stats are private; no private values are mounted.">
<meta name="prop:loading" content="bool" data-doc="Loading profile data; hides all stale identity, statistics and actions behind the shared Spinner.">
<meta name="prop:error" content="string" data-doc="Error message; hides stale data and offers retry. Empty means no error.">
<meta name="prop:empty" content="bool" data-doc="No profile found; hides stale identity, statistics and actions.">
<meta name="fixture:default" content='{"name":"Brenna Vale","handle":"@brenna","portrait":0,"level":12,"status":"online","rank":"Gold II · Season 4","bio":"Support main. Here for the team.","stats":[{"id":"wins","label":"Wins","value":"128","detail":"This season"},{"id":"matches","label":"Matches","value":"204","detail":"Ranked"},{"id":"best","label":"Best streak","value":"9","detail":"Consecutive wins"}],"actions":[{"id":"friend","label":"Add friend","disabled":false,"reason":"","pending":false},{"id":"invite","label":"Invite","disabled":true,"reason":"Player is in a match","pending":false}],"privateStats":false,"loading":false,"error":"","empty":false}'>
<meta name="fixture:private" content='{"privateStats":true,"status":"offline"}'>
<meta name="fixture:pending" content='{"actions":[{"id":"friend","label":"Add friend","disabled":false,"reason":"","pending":true}]}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:error" content='{"error":"Profile could not be loaded. Check your connection."}'>
<meta name="fixture:empty" content='{"empty":true}'>
<meta name="fixture:no-stats" content='{"stats":[],"actions":[]}'>
<meta name="fixture:overflow" content='{"name":"Maximilian Vandenberghe-Okonkwo","handle":"@the-unforgotten-champion-of-the-eastern-isles","rank":"Grandmaster · Northern Championship Series","bio":"A player biography that takes several lines on a phone and never hides the player actions or pushes statistics beyond the screen width.","stats":[{"id":"damage","label":"Highest accumulated damage in a single championship","value":"123,456,789,012","detail":"All seasons combined"},{"id":"wins","label":"Wins","value":"12,345","detail":"Across all regions"}],"actions":[{"id":"friend","label":"Send a companion request","disabled":false,"reason":"","pending":false},{"id":"invite","label":"Invite to the championship lobby","disabled":true,"reason":"This player does not accept invitations from non-friends","pending":false}]}'>

<section id="Profile" part="Profile.frame" data-layout="stack" data-gap="loose" aria-label={t("profile.title")} aria-busy={loading}>
  {#if loading}
    <div part="Profile.spinner" data-layout="row"><Spinner name={t("profile.loading")} /></div>
    <p part="Profile.state" role="status">{t("profile.loading")}</p>
  {:else if error}
    <p part="Profile.state" role="status">{error}</p>
    <button id="Retry" part="Profile.retry" onclick={retry}>{t("profile.retry")}</button>
  {:else if empty}
    <p part="Profile.state" role="status">{t("profile.empty")}</p>
  {:else}
    <div class="scroll-area" data-layout="overlay" data-gap="none">
    <div class="content" data-layout="scroll" data-gap="loose">
      <header part="Profile.identity" data-layout="row" data-gap="loose">
        <div part="Profile.avatar"><Avatar name={name || t("profile.player")} showName={false} portrait={portrait} level={level} status={status} /></div>
        <div class="identity-copy" data-layout="stack" data-gap="tight" data-fill>
          <h2 part="Profile.name Profile.title">{name || t("profile.player")}</h2>
          {#if handle}<div part="Profile.handle Profile.caption">{handle}</div>{/if}
          {#if rank}<div part="Profile.rank Profile.headline">{rank}</div>{/if}
        </div>
      </header>
      {#if bio}<p part="Profile.bio">{bio}</p>{/if}
      {#if privateStats}<p part="Profile.state">{t("profile.private")}</p>
      {:else if stats.length}
        <div part="Profile.stats" data-layout="grid" role="list" aria-label={t("profile.stats")}>
          {#each stats as s (s.id)}
            <div part="Profile.stat Profile.cell" data-layout="stack" data-gap="tight" role="listitem">
              <div part="Profile.label Profile.caption">{s.label}</div>
              <div part="Profile.value Profile.figure">{s.value}</div>
              {#if s.detail}<div part="Profile.detail Profile.caption">{s.detail}</div>{/if}
            </div>
          {/each}
        </div>
      {:else}<p part="Profile.state">{t("profile.noStats")}</p>{/if}
    </div>
    <div part="Profile.fade" data-anchor="bottom-edge"></div>
    </div>
    {#if actions.length}
      <footer part="Profile.actions" data-layout="switcher">
        {#each actions as a (a.id)}
          <div class="action-row" data-layout="stack" data-gap="tight" data-fill>
            <button id="Action" part="Profile.action" disabled={a.disabled || a.pending} aria-busy={a.pending} onclick={action}>{a.pending ? t("profile.sending") : a.label}</button>
            {#if a.reason}<div part="Profile.reason Profile.caption">{a.reason}</div>{/if}
          </div>
        {/each}
      </footer>
    {/if}
  {/if}
</section>
<style>
/* A player's profile on a smoked cut card: the name in condensed
   capitals beside the avatar, the rank on a gold tag, each figure on a
   cut plate under its label, the actions as plates lit along their top. */
:host { inline-size: stretch; max-block-size: stretch; color: var(--hud-ink); --fade-fill: var(--party-open-fill); }
[part="Profile.frame"] { box-sizing: border-box; padding: var(--space-lg); border: 0; border-radius: 0; color: var(--hud-ink); background: var(--party-card-fill); box-shadow: var(--party-card-rim); clip-path: polygon(var(--party-cut) 0, 100% 0, 100% calc(100% - var(--party-cut)), calc(100% - var(--party-cut)) 100%, 0 100%, 0 var(--party-cut)); }
.scroll-area { position: relative; min-block-size: 0; }
.content { padding-block-end: var(--space-lg); }
[part="Profile.identity"] { flex: none; }
[part="Profile.avatar"] { flex: none; }
.identity-copy { min-inline-size: 0; align-items: flex-start; }
[part~="Profile.name"] { margin: 0; max-inline-size: stretch; font-size: var(--party-size-name); line-height: 1.05; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: none; overflow-wrap: break-word; }
[part~="Profile.handle"] { max-inline-size: stretch; font-size: var(--hud-size-xs); color: var(--hud-ink-dim); overflow-wrap: anywhere; }
[part~="Profile.rank"] { max-inline-size: stretch; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.5; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--party-gold-ink); background: var(--hud-key-gold-fill); overflow-wrap: break-word; clip-path: polygon(var(--hud-tag-cut) 0, 100% 0, 100% calc(100% - var(--hud-tag-cut)), calc(100% - var(--hud-tag-cut)) 100%, 0 100%, 0 var(--hud-tag-cut)); }
[part="Profile.bio"] { margin: 0; font-size: var(--hud-size-name); line-height: 1.4; color: var(--hud-ink); overflow-wrap: break-word; }
[part="Profile.state"] { margin: 0; text-align: center; font-size: var(--hud-size-sm); color: var(--hud-ink-dim); overflow-wrap: break-word; }
/* As many plates a line as keep a long label's words whole at the player's text size. */
[part="Profile.stats"] { grid-template-columns: repeat(auto-fill, minmax(min(calc(200 * var(--font-step-px)), 100%), 1fr)); flex: none; }
[part~="Profile.stat"] { min-inline-size: 0; padding: var(--space-sm) var(--space-md); border: 0; border-radius: 0; color: var(--hud-ink); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); container-type: inline-size; clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
[part~="Profile.label"] { font-size: var(--hud-label-size); line-height: 1.3; font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); overflow-wrap: break-word; }
[part~="Profile.value"] { font-size: var(--party-size-stat); line-height: 1.05; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); text-shadow: none; overflow-wrap: break-word; }
[part~="Profile.detail"] { font-size: var(--hud-size-xs); color: var(--hud-ink-dim); overflow-wrap: break-word; }
[part="Profile.actions"] { flex: none; }
.action-row { min-inline-size: 0; }
[part="Profile.action"], [part="Profile.retry"] { min-height: var(--size-touch); gap: var(--space-xs); padding: 0 var(--space-lg); border: 0; border-radius: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--party-button-fill); box-shadow: var(--party-button-rim); text-shadow: none; clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); white-space: normal; text-align: center; padding-block: var(--space-xs); }
[part="Profile.action"]:hover, [part="Profile.retry"]:hover { background: var(--party-button-hover-fill); }
[part="Profile.action"]:focus-visible, [part="Profile.retry"]:focus-visible { box-shadow: var(--party-focus-rim); }
[part="Profile.action"]:disabled { opacity: 0.45; }
[part~="Profile.reason"] { text-align: center; font-size: var(--hud-size-xs); color: var(--hud-ink-dim); overflow-wrap: break-word; }
[part="Profile.spinner"] { justify-content: center; }
@media (width < 600px) {
  [part="Profile.frame"] { padding: var(--space-sm); }
  [part="Profile.stats"] { grid-template-columns: minmax(0, 1fr); --layout-gap: var(--layout-gap-tight); }
  [part~="Profile.stat"] { padding: var(--space-xs) var(--space-md); container-type: normal; }
  [part~="Profile.value"] { font-size: var(--party-size-figure-compact); }
  [part~="Profile.name"] { font-size: var(--party-size-name-compact); }
}
</style>
