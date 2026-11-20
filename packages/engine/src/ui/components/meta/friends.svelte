<meta name="component" content="Friends">
<meta name="strings" content="./general.en.ogt">
<meta name="prop:title" content="string" data-doc="The panel's heading; empty takes the group's (Friends).">
<meta name="prop:requests" content="list" data-doc="Friend requests waiting for an answer: keyed rows of { id, name, avatar, note }: avatar an icon of the style's atlas; note a second line as the game words it (3 mutual friends), empty hides it. Empty hides the section.">
<meta name="prop:friends" content="list" data-doc="The friends in the game's order: keyed rows of { id, name, avatar, status, activity, canInvite, invited }: status online, ingame, away or offline, shown in words and by a mark; activity what they are doing (Ranked match, Last seen 2 d ago), empty hides it; canInvite the game will send an invite; invited one is pending.">
<meta name="prop:selected" content="string" data-doc="The friend the game marks as chosen (data-selected, aria-current); empty is none.">
<meta name="fixture:default" content='{"title":"","requests":[{"id":"r1","name":"Idris","avatar":"person","note":"2 mutual friends"}],"friends":[{"id":"aerin","name":"Aerin","avatar":"person","status":"ingame","activity":"Ranked match","canInvite":false,"invited":false},{"id":"lys","name":"Lys","avatar":"person","status":"online","activity":"In the lobby","canInvite":true,"invited":false},{"id":"maren","name":"Maren","avatar":"person","status":"away","activity":"","canInvite":true,"invited":true},{"id":"sela","name":"Sela","avatar":"person","status":"offline","activity":"Last seen 2 d ago","canInvite":false,"invited":false}],"selected":"lys"}'>
<meta name="fixture:empty" content='{"title":"","requests":[],"friends":[],"selected":""}'>
<meta name="fixture:requests" content='{"title":"","requests":[{"id":"r1","name":"Idris","avatar":"person","note":"2 mutual friends"},{"id":"r2","name":"Ysolde","avatar":"person","note":""}],"friends":[],"selected":""}'>
<meta name="fixture:overflow" content='{"title":"Companions and Allies Across the Realm","requests":[{"id":"r1","name":"Bartholomew Fitzgerald-Montgomery the Third","avatar":"person","note":"Sent you a request a very long time ago, with many mutual friends"}],"friends":[{"id":"f1","name":"Maximilian the Unbreakable Lord of the Northern Wastes","avatar":"person","status":"ingame","activity":"Ranked match — Season 12 grand final, round 3 of 5","canInvite":false,"invited":false},{"id":"f2","name":"Ana","avatar":"person","status":"online","activity":"","canInvite":true,"invited":false},{"id":"f3","name":"Bo","avatar":"person","status":"online","activity":"Lobby","canInvite":true,"invited":false},{"id":"f4","name":"Cy","avatar":"person","status":"away","activity":"","canInvite":true,"invited":false},{"id":"f5","name":"Di","avatar":"person","status":"offline","activity":"Last seen 1 h ago","canInvite":false,"invited":false},{"id":"f6","name":"Ed","avatar":"person","status":"offline","activity":"Last seen 3 d ago","canInvite":false,"invited":false},{"id":"f7","name":"Flo","avatar":"person","status":"offline","activity":"","canInvite":false,"invited":false},{"id":"f8","name":"Gus","avatar":"person","status":"offline","activity":"","canInvite":false,"invited":false}],"selected":"f2"}'>

<!--
  The friends panel: a band with the title and Add friend, the friend
  requests with Accept and Decline, then the friends with an avatar, the
  name, the status in words beside a mark (online, in game, away, offline),
  what they are doing and Invite. The game owns the list, presence and
  whether an invite may go; the panel reports add, accept and decline (a
  request's key), choose and invite (a friend's key). It fills its height
  and scrolls inside it.
-->
<section id="Friends" part="frame" layout="stack" fill aria-label={title != '' ? title : t("general.friends.name")}>
  <header class="band" part="band" layout="wrap" gap="loose">
    <h2 class="title" part="title" fill>{title != '' ? title : t("general.friends.name")}</h2>
    <button id="Add" class="secondary" onclick={add}><og-icon class="mark" icon="plus"></og-icon><span>{t("general.friends.add")}</span></button>
  </header>
  <div class="body" layout="scroll" gap="loose">
    {#if requests.length}<section class="section" layout="stack" gap="tight" aria-label={t("general.friends.requests")}>
      <h3 class="heading" part="heading">{t("general.friends.requests")}</h3>
      <ul class="rows" layout="stack" gap="hair">
        {#each requests as r (r.id)}
          <li class="request row" part="row" layout="wrap" aria-label={r.name}>
            <div class="avatar picture" part="well" layout="row" gap="none"><og-icon class="avatar-icon" icon={r.avatar}></og-icon></div>
            <div class="who" layout="stack" gap="none" fill>
              <p class="name" part="headline">{r.name}</p>
              {#if r.note}<span part="caption" class="note">{r.note}</span>{/if}
            </div>
            <div class="actions" layout="row" gap="tight">
              <button id="Accept" tone="confirm" class="go" variant="primary" aria-label={t("general.friends.acceptName", { name: r.name })} onclick={accept}><og-icon class="mark" icon="check"></og-icon><span class="action-label">{t("general.friends.accept")}</span></button>
              <button id="Decline" variant="ghost" aria-label={t("general.friends.declineName", { name: r.name })} onclick={decline}><og-icon class="mark" icon="close"></og-icon><span class="action-label">{t("general.friends.decline")}</span></button>
            </div>
          </li>
        {/each}
      </ul>
    </section>{/if}
    <section class="section" layout="stack" gap="tight" aria-label={t("general.friends.list")}>
      {#if requests.length && friends.length}<h3 class="heading" part="heading">{t("general.friends.list")}</h3>{/if}
      {#if friends.length}
        <ul class="rows" layout="stack" gap="hair" aria-label={t("general.friends.list")}>
          {#each friends as f (f.id)}
            <li class="friend" layout="row">
              <button class="row" part="row" fill aria-current={f.id == selected ? 'true' : 'false'} aria-label={f.name} onclick={choose} state:selected={f.id == selected} class:status-away={(f.status) == 'away'} class:status-ingame={(f.status) == 'ingame'} class:status-offline={(f.status) == 'offline'} class:status-online={(f.status) == 'online'}><div class="face" layout="row">
                <div class="avatar-wrap" class:status-away={(f.status) == 'away'} class:status-ingame={(f.status) == 'ingame'} class:status-offline={(f.status) == 'offline'} class:status-online={(f.status) == 'online'}><div class="avatar picture" part="well" layout="row" gap="none" class:status-away={(f.status) == 'away'} class:status-ingame={(f.status) == 'ingame'} class:status-offline={(f.status) == 'offline'} class:status-online={(f.status) == 'online'}><og-icon class="avatar-icon" icon={f.avatar}></og-icon></div><div part="dot" class="presence well" tone={f.status == 'online' ? 'success' : f.status == 'ingame' ? 'info' : f.status == 'away' ? 'warn' : ''} class:status-away={(f.status) == 'away'} class:status-ingame={(f.status) == 'ingame'} class:status-offline={(f.status) == 'offline'} class:status-online={(f.status) == 'online'}></div></div>
                <div class="who" layout="stack" gap="none" fill>
                  <p class="name" part="headline">{f.name}</p>
                  <div class="line" layout="row"><p class="status" part="label" class:status-away={(f.status) == 'away'} class:status-ingame={(f.status) == 'ingame'} class:status-offline={(f.status) == 'offline'} class:status-online={(f.status) == 'online'}>{f.status == 'ingame' ? t("general.friends.ingame") : f.status == 'online' ? t("general.friends.online") : f.status == 'away' ? t("general.friends.away") : t("general.friends.offline")}</p>{#if f.activity}<span part="caption" class="activity">{f.activity}</span>{/if}</div>
                </div>
              </div></button>
              {#if f.canInvite || f.invited}<button id="Invite" class="secondary" data-invited={f.invited} disabled={f.invited} aria-label={f.invited ? t("general.friends.invitedName", { name: f.name }) : t("general.friends.inviteName", { name: f.name })} onclick={invite}><og-icon class="mark" icon={f.invited ? 'clock' : 'plus'}></og-icon><span class="action-label">{f.invited ? t("general.friends.invited") : t("general.friends.invite")}</span></button>{/if}
            </li>
          {/each}
        </ul>
      {:else}
        <div class="empty" layout="stack" role="status"><og-icon class="empty-mark" icon="person"></og-icon><p class="empty-text">{t("general.friends.empty")}</p></div>
      {/if}
    </section>
  </div>
</section>

<style>
#Friends p { margin: 0; }
.band { width: stretch; flex: none; }
/* The title keeps its words whole and takes its own width first: Add
   friend wraps under it when both do not fit. */
.title { margin: 0; flex: 1 1 auto; }
#Add { flex: none; min-height: var(--size-touch); }
.mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
.section { width: stretch; }
/* A section's heading: gold small capitals after a short gold rule. */
.heading { margin: 0; padding: var(--space-xxs) var(--space-xs); font: var(--caption-text); font-weight: var(--quest-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--quest-gold); }
.rows { width: stretch; margin: 0; padding: 0; list-style: none; }
.request, .friend { width: stretch; }
/* A request: a cut card washed gold, Accept on the reward's gold face. */
.request { padding: var(--space-xs) var(--space-sm); background: var(--quest-ready-fill); box-shadow: var(--quest-ready-rim); border-radius: var(--quest-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
/* A friend: a cut row; the one chosen washed gold with a gold bar. */
button.row { min-height: var(--size-touch); padding: var(--space-xs) var(--space-sm); justify-content: flex-start; background: none; box-shadow: none; border-radius: var(--quest-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; --button-radius: var(--quest-cut) 0; --button-corner-shape: var(--cut-shape); }
button.row:hover { background: var(--quest-entry-fill); }
button.row:state(selected) { background: var(--quest-selected-fill); box-shadow: var(--quest-selected-rim); }
.row .face { width: stretch; }
/* The avatar in a round socket with a lit edge (a pack squares it through
   its avatar radius); gold on a request. */
.avatar { flex: none; width: var(--size-lg); height: var(--size-lg); justify-content: center; background: var(--quest-socket-fill); box-shadow: var(--quest-well-rim); --well-radius: var(--avatar-radius); --picture-radius: var(--avatar-radius); color: var(--ink-bright); }
.request .avatar { background: var(--quest-socket-gold-fill); box-shadow: var(--quest-well-ready-rim); color: var(--quest-gold); }
.avatar-icon { width: var(--size-sm); height: var(--size-sm); }
.avatar-wrap { position: relative; flex: none; }
/* Presence: a diamond in its tone at the socket's foot. */
.presence { position: absolute; inset-inline-end: calc(-1 * var(--stroke-thin)); bottom: calc(-1 * var(--stroke-thin)); width: var(--space-sm); height: var(--space-sm); border-radius: 0; rotate: 45deg; --dot-fill: currentColor; box-shadow: var(--quest-well-rim); }
.presence.status-offline { --dot-fill: var(--ink-disabled); }
/* Away or offline: the avatar and its mark dim; the name keeps its ink. */
.avatar-wrap.status-away .avatar, .avatar-wrap.status-offline .avatar, .avatar-wrap.status-offline .presence { opacity: var(--social-off-opacity); }
.name { text-align: start; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: var(--ink-bright); }
.line { width: stretch; align-items: baseline; }
.status { flex: none; white-space: nowrap; font-weight: var(--quest-weight); }
.status.status-online { color: var(--tone-success); }
.status.status-ingame { color: var(--tone-info); }
.status.status-away { color: var(--alert-warn); }
.status.status-offline { color: var(--quest-ink-quiet); }
.activity, .note { min-width: 0; text-align: start; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: var(--quest-ink-quiet); }
.actions { flex: none; margin-inline-start: auto; }
.actions button, #Invite { flex: none; min-height: var(--size-touch); min-width: var(--size-touch); }
.empty { width: stretch; align-items: center; padding: var(--space-xl) 0; color: var(--quest-ink-quiet); }
.empty-mark { width: var(--size-lg); height: var(--size-lg); }
.empty-text { margin: 0; }
@container (width < 400px) { .action-label { display: none; } }
</style>
