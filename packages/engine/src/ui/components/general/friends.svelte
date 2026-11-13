<meta name="component" content="Friends">
<meta name="strings" content="./general.en.ogt">
<meta name="parts" content="Friends.frame Friends.band Friends.title Friends.heading Friends.request Friends.row Friends.avatar Friends.presence Friends.name Friends.status">
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
<section id="Friends" part="Friends.frame" aria-label={title != '' ? title : t("general.friends.name")}>
  <header part="Friends.band">
    <h2 part="Friends.title">{title != '' ? title : t("general.friends.name")}</h2>
    <button id="Add" class="secondary" onclick={add}><og-icon class="mark" icon="plus"></og-icon><span>{t("general.friends.add")}</span></button>
  </header>
  <scroll class="body">
    <section class="section" aria-label={t("general.friends.requests")} hidden={!requests.length}>
      <h3 part="Friends.heading">{t("general.friends.requests")}</h3>
      <ul class="rows">
        {#each requests as r (r.id)}
          <li part="Friends.request" aria-label={r.name}>
            <div part="Friends.avatar"><og-icon class="avatar-icon" icon={r.avatar}></og-icon></div>
            <div class="who">
              <p part="Friends.name">{r.name}</p>
              <span class="note" hidden={r.note == ''}>{r.note}</span>
            </div>
            <div class="actions">
              <button id="Accept" variant="primary" aria-label={t("general.friends.acceptName", { name: r.name })} onclick={accept}><og-icon class="mark" icon="check"></og-icon><span class="action-label">{t("general.friends.accept")}</span></button>
              <button id="Decline" variant="ghost" aria-label={t("general.friends.declineName", { name: r.name })} onclick={decline}><og-icon class="mark" icon="close"></og-icon><span class="action-label">{t("general.friends.decline")}</span></button>
            </div>
          </li>
        {/each}
      </ul>
    </section>
    <section class="section" aria-label={t("general.friends.list")}>
      <h3 part="Friends.heading" hidden={!requests.length || !friends.length}>{t("general.friends.list")}</h3>
      {#if friends.length}
        <ul class="rows" aria-label={t("general.friends.list")}>
          {#each friends as f (f.id)}
            <li class="friend">
              <button part="Friends.row" data-status={f.status} data-selected={f.id == selected} aria-current={f.id == selected ? 'true' : 'false'} aria-label={f.name} onclick={choose}><div class="face">
                <div part="Friends.avatar" data-status={f.status}><og-icon class="avatar-icon" icon={f.avatar}></og-icon><div part="Friends.presence" data-status={f.status}></div></div>
                <div class="who">
                  <p part="Friends.name">{f.name}</p>
                  <div class="line"><p part="Friends.status" data-status={f.status}>{f.status == 'ingame' ? t("general.friends.ingame") : f.status == 'online' ? t("general.friends.online") : f.status == 'away' ? t("general.friends.away") : t("general.friends.offline")}</p><span class="activity" hidden={f.activity == ''}>{f.activity}</span></div>
                </div>
              </div></button>
              <button id="Invite" class="secondary" data-invited={f.invited} hidden={!f.canInvite && !f.invited} disabled={f.invited} aria-label={f.invited ? t("general.friends.invitedName", { name: f.name }) : t("general.friends.inviteName", { name: f.name })} onclick={invite}><og-icon class="mark" icon={f.invited ? 'clock' : 'plus'}></og-icon><span class="action-label">{f.invited ? t("general.friends.invited") : t("general.friends.invite")}</span></button>
            </li>
          {/each}
        </ul>
      {:else}
        <div class="empty" role="status"><og-icon class="empty-mark" icon="person"></og-icon><p class="empty-text">{t("general.friends.empty")}</p></div>
      {/if}
    </section>
  </scroll>
</section>

<style>
[part="Friends.frame"] p { margin: 0; }
[part="Friends.frame"][hidden], [part="Friends.frame"] [hidden] { display: none; }
[part="Friends.frame"] { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-sm); container-type: inline-size; }
[part="Friends.band"] { display: flex; width: stretch; align-items: center; gap: var(--space-xs) var(--space-md); flex: none; flex-wrap: wrap; }
[part="Friends.title"] { margin: 0; flex: 1; min-width: 0; }
#Add { flex: none; margin-inline-start: auto; min-height: var(--size-touch); gap: var(--space-xs); }
.mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
.body { width: stretch; flex: 1; min-height: 0; gap: var(--space-md); }
.section { display: flex; flex-direction: column; width: stretch; gap: var(--space-xs); }
[part="Friends.heading"] { margin: 0; padding: var(--space-xxs) var(--space-sm); }
.rows { display: flex; flex-direction: column; width: stretch; margin: 0; padding: 0; list-style: none; gap: var(--space-xxs); }
[part="Friends.request"], .friend { display: flex; width: stretch; align-items: center; gap: var(--space-sm); }
[part="Friends.request"] { padding: var(--space-xs) var(--space-sm); flex-wrap: wrap; }
[part="Friends.row"] { width: stretch; min-width: 0; min-height: var(--size-touch); padding: var(--space-xs) var(--space-sm); justify-content: flex-start; }
[part="Friends.row"] .face { display: flex; width: stretch; align-items: center; gap: var(--space-sm); }
[part="Friends.avatar"] { position: relative; display: flex; flex: none; width: var(--size-lg); height: var(--size-lg); align-items: center; justify-content: center; }
.avatar-icon { width: var(--size-sm); height: var(--size-sm); }
[part="Friends.presence"] { position: absolute; inset-inline-end: 0; bottom: 0; width: var(--space-md); height: var(--space-md); }
.who { display: flex; flex-direction: column; width: stretch; min-width: 0; gap: 0; flex: 1; }
[part="Friends.name"] { text-align: start; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.line { display: flex; width: stretch; min-width: 0; gap: var(--space-sm); align-items: baseline; }
[part="Friends.status"] { flex: none; white-space: nowrap; }
.activity, .note { min-width: 0; text-align: start; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.actions { display: flex; flex: none; gap: var(--space-xs); margin-inline-start: auto; }
.actions button, #Invite { flex: none; min-height: var(--size-touch); min-width: var(--size-touch); gap: var(--space-xs); }
.empty { display: flex; flex-direction: column; width: stretch; gap: var(--space-sm); align-items: center; justify-content: center; padding: var(--space-xl) 0; }
.empty-mark { width: var(--size-lg); height: var(--size-lg); }
.empty-text { margin: 0; }
@container (width < 400px) { .action-label { display: none; } }
</style>
