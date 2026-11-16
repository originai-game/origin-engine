<meta name="component" content="Profile">
<meta name="strings" content="./profile.en.ogt">
<meta name="parts" content="Profile.frame Profile.identity Profile.avatar Profile.name Profile.handle Profile.rank Profile.bio Profile.stats Profile.stat Profile.label Profile.value Profile.detail Profile.actions Profile.action Profile.reason Profile.state Profile.spinner Profile.retry">
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

<section id="Profile" part="Profile.frame" aria-label={t("profile.title")} aria-busy={loading}>
  {#if loading}
    <div part="Profile.spinner"><Spinner name={t("profile.loading")} /></div>
    <p part="Profile.state" role="status">{t("profile.loading")}</p>
  {:else if error}
    <p part="Profile.state" role="status">{error}</p>
    <button id="Retry" part="Profile.retry" onclick={retry}>{t("profile.retry")}</button>
  {:else if empty}
    <p part="Profile.state" role="status">{t("profile.empty")}</p>
  {:else}
    <div class="scroll-area">
    <div class="content">
      <header part="Profile.identity">
        <div part="Profile.avatar"><Avatar name={name || t("profile.player")} showName={false} portrait={portrait} level={level} status={status} /></div>
        <div class="identity-copy">
          <h2 part="Profile.name">{name || t("profile.player")}</h2>
          {#if handle}<div part="Profile.handle">{handle}</div>{/if}
          {#if rank}<div part="Profile.rank">{rank}</div>{/if}
        </div>
      </header>
      {#if bio}<p part="Profile.bio">{bio}</p>{/if}
      {#if privateStats}<p part="Profile.state">{t("profile.private")}</p>
      {:else if stats.length}
        <div part="Profile.stats" role="list" aria-label={t("profile.stats")}>
          {#each stats as s (s.id)}
            <div part="Profile.stat" role="listitem">
              <div part="Profile.label">{s.label}</div>
              <div part="Profile.value">{s.value}</div>
              {#if s.detail}<div part="Profile.detail">{s.detail}</div>{/if}
            </div>
          {/each}
        </div>
      {:else}<p part="Profile.state">{t("profile.noStats")}</p>{/if}
    </div>
    <div class="scroll-fade"></div>
    </div>
    {#if actions.length}
      <footer part="Profile.actions">
        {#each actions as a (a.id)}
          <div class="action-row">
            <button id="Action" part="Profile.action" disabled={a.disabled || a.pending} aria-busy={a.pending} onclick={action}>{a.pending ? t("profile.sending") : a.label}</button>
            {#if a.reason}<div part="Profile.reason">{a.reason}</div>{/if}
          </div>
        {/each}
      </footer>
    {/if}
  {/if}
</section>
<style>
:host { display: flex; flex-direction: column; inline-size: stretch; max-block-size: stretch; gap: var(--space-md); }
.scroll-area { display: flex; flex-direction: column; position: relative; min-block-size: 0; }
.content { display: flex; flex-direction: column; min-block-size: 0; overflow-y: auto; gap: var(--space-md); padding-block-end: var(--space-lg); }
.scroll-fade { position: absolute; inset-inline: 0; bottom: 0; block-size: var(--space-lg); pointer-events: none; background: linear-gradient(transparent, var(--panel)); }
[part="Profile.identity"] { display: flex; align-items: center; gap: var(--space-md); flex: none; }
[part="Profile.avatar"] { flex: none; }
.identity-copy { display: flex; flex-direction: column; gap: var(--space-xs); min-inline-size: 0; }
[part="Profile.name"], [part="Profile.bio"], [part="Profile.state"] { margin: 0; }
[part="Profile.name"], [part="Profile.handle"], [part="Profile.rank"], [part="Profile.bio"], [part="Profile.label"], [part="Profile.value"], [part="Profile.detail"], [part="Profile.reason"], [part="Profile.state"] { overflow-wrap: anywhere; }
[part="Profile.handle"], [part="Profile.detail"], [part="Profile.reason"] { color: var(--ink-muted); font: var(--text-sm); }
[part="Profile.rank"] { font-weight: var(--font-weight-semibold); }
[part="Profile.stats"] { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--space-sm); flex: none; }
[part="Profile.stat"] { display: flex; flex-direction: column; gap: var(--space-xs); padding: var(--space-sm); border: var(--stroke-hairline) solid var(--line); border-radius: var(--radius-control); }
[part="Profile.label"] { color: var(--ink-muted); font: var(--text-sm); }
[part="Profile.value"] { font-family: var(--figure-font); font-weight: var(--font-weight-semibold); }
[part="Profile.actions"] { display: flex; flex-wrap: wrap; gap: var(--space-sm); flex: none; }
.action-row { display: flex; flex-direction: column; flex: 1; min-inline-size: 0; gap: var(--space-xs); }
[part="Profile.action"], [part="Profile.retry"] { min-block-size: var(--size-touch); }
[part="Profile.spinner"] { display: flex; justify-content: center; }
@media (width < 600px) { [part="Profile.stats"] { grid-template-columns: 1fr; } [part="Profile.actions"] { flex-direction: column; } }
</style>
