<meta name="component" content="Chat">
<meta name="strings" content="./general.en.ogt">
<meta name="prop:channels" content="list" data-doc="The channels, keyed rows of { id, label, unread }: unread the count the game keeps, 0 hides its badge. Empty shows no tab row.">
<meta name="prop:channel" content="string" data-doc="The channel shown.">
<meta name="prop:messages" content="list" data-doc="The shown channel's messages, oldest first: keyed rows of { id, kind, sender, time, text }: kind player, own (the player's) or system; time as the game formats it; text wraps, and a long word breaks.">
<meta name="prop:draft" content="string" data-doc="The message being written, two-way: bind a writable store (bind:draft={$draft}) and the field writes what is typed back to it; the game clears it after a send.">
<meta name="prop:maxLength" content="number" data-doc="The longest message the game takes, in characters.">
<meta name="prop:canSend" content="bool" data-doc="The game takes a message now; false disables the field and Send (muted, slow mode).">
<meta name="prop:notice" content="string" data-doc="A line over the field as the game words it (Slow mode: 3 s, You are muted); empty hides it.">
<meta name="fixture:default" content='{"channels":[{"id":"world","label":"World","unread":0},{"id":"guild","label":"Guild","unread":3},{"id":"party","label":"Party","unread":0}],"channel":"world","messages":[{"id":"m1","kind":"system","sender":"","time":"20:14","text":"Welcome to the World channel. Be kind."},{"id":"m2","kind":"player","sender":"Aerin","time":"20:15","text":"Anyone up for the raid tonight?"},{"id":"m3","kind":"own","sender":"Brenna","time":"20:16","text":"Count me in, I will bring potions."},{"id":"m4","kind":"player","sender":"Lys","time":"20:16","text":"Same here!"}],"draft":"","maxLength":200,"canSend":true,"notice":""}'>
<meta name="fixture:empty" content='{"channels":[{"id":"world","label":"World","unread":0}],"channel":"world","messages":[],"draft":"","maxLength":200,"canSend":true,"notice":""}'>
<meta name="fixture:disabled" content='{"channels":[{"id":"world","label":"World","unread":0}],"channel":"world","messages":[{"id":"m1","kind":"system","sender":"","time":"20:20","text":"You are muted for 5 minutes."}],"draft":"","maxLength":200,"canSend":false,"notice":"You are muted for 5 minutes."}'>
<meta name="fixture:overflow" content='{"channels":[{"id":"world","label":"World","unread":0},{"id":"guild","label":"The Order of the Silver Dawn","unread":128},{"id":"party","label":"Party","unread":9},{"id":"trade","label":"Trade","unread":0},{"id":"whisper","label":"Whispers","unread":1}],"channel":"guild","messages":[{"id":"m1","kind":"system","sender":"","time":"19:58","text":"Guild message of the day: the raid starts at nine, bring your best gear and enough food for three hours."},{"id":"m2","kind":"player","sender":"Maximilian the Unbreakable","time":"20:01","text":"Supercalifragilisticexpialidocious-wordthatdoesnotbreaknaturallyanywhereatallsoitmustbreak"},{"id":"m3","kind":"own","sender":"Brenna","time":"20:02","text":"A rather long message from me that wraps across several lines because I had a lot to say about the raid plan and the loot rules."},{"id":"m4","kind":"player","sender":"Lys","time":"20:03","text":"ok"},{"id":"m5","kind":"player","sender":"Aerin","time":"20:04","text":"See you there."},{"id":"m6","kind":"player","sender":"Idris","time":"20:05","text":"Who has the key?"},{"id":"m7","kind":"own","sender":"Brenna","time":"20:05","text":"I do."},{"id":"m8","kind":"player","sender":"Maren","time":"20:06","text":"Great, then we are set."},{"id":"m9","kind":"system","sender":"","time":"20:07","text":"Sela joined the guild."},{"id":"m10","kind":"player","sender":"Sela","time":"20:07","text":"Hello everyone!"}],"draft":"A draft that is already fairly long and keeps going","maxLength":200,"canSend":true,"notice":"Slow mode: one message every 3 s"}'>

<!--
  A chat panel: channel tabs with unread badges, the message log (system
  lines, the player's own messages and others' with sender and time, long
  text wrapping and long words breaking) and an input row. The game owns
  the channels, the messages, moderation and the clock; the panel reports
  choosetab (a channel's key) and send
  (the Send button), and writes the draft back to its store.
  The log fills the height it is given and scrolls inside it, newest at
  the foot.
-->
<section id="Chat" part="frame" layout="stack" fill aria-label={t("general.chat.name")}>
  {#if channels.length}<div class="tabs" role="tablist" aria-label={t("general.chat.channels")}>
    <div class="tabs-scroll">
      <div class="tabs-row" layout="row" gap="hair">
        {#each channels as c (c.id)}
          <button class="tab" part="tab" variant="quiet" role="tab" aria-selected={c.id == channel} aria-label={c.unread > 0 ? t("general.chat.unread", { channel: c.label, count: c.unread }) : c.label} onclick={choosetab} state:selected={c.id == channel}><div layout="row" gap="tight"><span class="tab-label">{c.label}</span>{#if c.unread > 0}<p class="unread" part="badge">{formatNumber(c.unread)}</p>{/if}</div></button>
        {/each}
      </div>
    </div>
  </div>{/if}
  <div part="well" class="well" layout="stack" gap="none" fill><div class="log" part="log" layout="scroll" reverse gap="none" fill role="log" aria-label={t("general.chat.messages")} aria-live="polite">
    {#if messages.length}
      <ol class="messages" layout="stack" gap="tight">
        {#each messages as m (m.id)}
          <li class="message" part="message" layout="stack" gap="hair" class:kind-own={(m.kind) == 'own'} class:kind-system={(m.kind) == 'system'}>
            {#if m.kind != 'system'}<div class="head" layout="row">
              <p class="sender" part="label" class:kind-own={(m.kind) == 'own'} class:kind-system={(m.kind) == 'system'}>{m.kind == 'own' ? t("general.chat.you") : m.sender}</p>
              <p class="time figure" part="caption">{m.time}</p>
            </div>{:else}<div class="head" layout="row">
              <og-icon class="mark" icon="bolt"></og-icon>
              <p class="sender" part="label" class:kind-system={true}>{t("general.chat.system")}</p>
              <p class="time figure" part="caption">{m.time}</p>
            </div>{/if}
            <p class="text" part="text" class:bubble={m.kind != 'system'} class:kind-own={(m.kind) == 'own'} class:kind-system={(m.kind) == 'system'}>{m.text}</p>
          </li>
        {/each}
      </ol>
    {:else}
      <p class="empty" role="status">{t("general.chat.empty")}</p>
    {/if}
  </div></div>
  {#if notice}<p class="notice" part="label" role="status">{notice}</p>{/if}
  <form class="compose" part="compose" layout="row" gap="tight" onsubmit={send}>
    <input id="Draft" fill type="text" aria-label={t("general.chat.placeholder")} placeholder={t("general.chat.placeholder")} maxlength={maxLength} disabled={!canSend} bind:value={draft}>
    <button id="Send" class="go" variant="primary" aria-label={t("general.chat.send")} disabled={!canSend}><div layout="row" gap="tight"><og-icon class="send-mark" icon="play"></og-icon><span class="send-label">{t("general.chat.send")}</span></div></button>
  </form>
</section>

<style>
.tabs { width: stretch; flex: none; padding-block-end: var(--space-xxs); }
.tabs-scroll { width: stretch; overflow-x: auto; overflow-y: hidden; min-width: 0; }
/* A channel: a cut tab; the one shown washed and lit gold along its foot. */
.tab { flex: none; min-height: var(--size-touch); padding: 0 var(--space-md); color: var(--quest-ink-quiet); border-radius: var(--quest-cut-sm) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.tab:state(selected) { color: var(--ink-bright); background: var(--social-tab-on-fill); box-shadow: var(--social-tab-on-rim); }
.tab-label { white-space: nowrap; max-width: calc(var(--size-xxl) * 2.5); overflow: hidden; text-overflow: ellipsis; font-weight: var(--quest-weight); }
/* Unread: a gold cut chip with a dark figure. */
.unread { margin: 0; min-width: var(--size-xs); padding: 0 var(--space-xxs); text-align: center; color: var(--quest-gold-ink); background: var(--quest-gold); font-weight: var(--hud-figure-weight); font-variant-numeric: var(--hud-figure-numeric); border-radius: var(--quest-cut-sm) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
/* The log in a dark cut well; the log scrolls inside the well's rim. */
.log { width: stretch; }
.well { width: stretch; min-height: 0; padding: var(--space-xs) var(--stroke-thin); background: var(--quest-well-fill); box-shadow: var(--quest-well-rim); border-radius: var(--quest-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.messages { width: stretch; margin: 0; padding: var(--space-sm); list-style: none; }
.message { width: stretch; padding: var(--space-xxs) 0; }
.message.kind-own { align-items: flex-end; }
.head { width: stretch; align-items: baseline; }
.message.kind-own .head { justify-content: flex-end; }
.sender { margin: 0; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-weight: var(--quest-weight); color: var(--social-sender-ink); }
.sender.kind-own { color: var(--quest-gold); }
.sender.kind-system { color: var(--quest-ink-quiet); }
.time { margin: 0; flex: none; white-space: nowrap; color: var(--quest-ink-faint); font-variant-numeric: var(--hud-figure-numeric); }
.mark { width: var(--size-xs); height: var(--size-xs); flex: none; color: var(--quest-gold); }
/* A line on a cut chip; the player's own washed gold at the far side; a system line bare and quiet. */
.text { margin: 0; text-align: start; overflow-wrap: anywhere; color: var(--ink-bright); }
.text.bubble { max-width: 88%; padding: var(--space-xs) var(--space-sm); background: var(--social-line-fill); border-radius: var(--quest-cut-sm) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.message.kind-own .text { text-align: end; background: var(--social-own-fill); box-shadow: var(--social-own-rim); }
.message.kind-system .text { max-width: stretch; padding: 0; color: var(--quest-ink-quiet); }
.empty { margin: 0; padding: var(--space-lg); text-align: center; color: var(--quest-ink-quiet); }
.notice { margin: 0; flex: none; color: var(--alert-warn); }
.compose { width: stretch; align-items: stretch; flex: none; }
#Draft { min-height: var(--size-touch); }
#Send { flex: none; min-height: var(--size-touch); min-width: var(--size-touch); }
.send-mark { width: var(--size-xs); height: var(--size-xs); }
.send-mark:dir(rtl) { transform: scaleX(-1); }
@container (width < 400px) { .send-label { display: none; } }
</style>
