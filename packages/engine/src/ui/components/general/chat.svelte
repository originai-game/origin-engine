<meta name="component" content="Chat">
<meta name="strings" content="./general.en.ogt">
<meta name="parts" content="Chat.frame Chat.tab Chat.unread Chat.log Chat.message Chat.sender Chat.time Chat.text Chat.notice Chat.compose">
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
<section id="Chat" part="Chat.frame" aria-label={t("general.chat.name")}>
  <div class="tabs" role="tablist" aria-label={t("general.chat.channels")} hidden={!channels.length}>
    <scroll class="tabs-scroll">
      <div class="tabs-row">
        {#each channels as c (c.id)}
          <button part="Chat.tab" variant="quiet" role="tab" aria-selected={c.id == channel} aria-label={c.unread > 0 ? t("general.chat.unread", { channel: c.label, count: c.unread }) : c.label} onclick={choosetab}><span class="tab-label">{c.label}</span><p part="Chat.unread" hidden={!(c.unread > 0)}>{formatNumber(c.unread)}</p></button>
        {/each}
      </div>
    </scroll>
  </div>
  <div part="Chat.log" role="log" aria-label={t("general.chat.messages")} aria-live="polite">
    {#if messages.length}
      <ol class="messages">
        {#each messages as m (m.id)}
          <li part="Chat.message" data-kind={m.kind}>
            <div class="head" hidden={m.kind == 'system'}>
              <p part="Chat.sender" data-kind={m.kind}>{m.kind == 'own' ? t("general.chat.you") : m.sender}</p>
              <p part="Chat.time">{m.time}</p>
            </div>
            <div class="head" hidden={m.kind != 'system'}>
              <og-icon class="mark" icon="bolt"></og-icon>
              <p part="Chat.sender" data-kind="system">{t("general.chat.system")}</p>
              <p part="Chat.time">{m.time}</p>
            </div>
            <p part="Chat.text" data-kind={m.kind}>{m.text}</p>
          </li>
        {/each}
      </ol>
    {:else}
      <p class="empty" role="status">{t("general.chat.empty")}</p>
    {/if}
  </div>
  <p part="Chat.notice" role="status" hidden={notice == ''}>{notice}</p>
  <form part="Chat.compose" onsubmit={send}>
    <input id="Draft" type="text" aria-label={t("general.chat.placeholder")} placeholder={t("general.chat.placeholder")} maxlength={maxLength} disabled={!canSend} bind:value={draft}>
    <button id="Send" variant="primary" aria-label={t("general.chat.send")} disabled={!canSend}><og-icon class="send-mark" icon="play"></og-icon><span class="send-label">{t("general.chat.send")}</span></button>
  </form>
</section>

<style>
[part="Chat.frame"][hidden], [part="Chat.frame"] [hidden] { display: none; }
[part="Chat.frame"] { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-xs); container-type: inline-size; }
.tabs { display: flex; width: stretch; flex: none; }
.tabs-scroll { width: stretch; overflow-x: auto; overflow-y: hidden; }
.tabs-row { display: flex; gap: var(--space-xxs); }
[part="Chat.tab"] { flex: none; min-height: var(--size-touch); padding: 0 var(--space-sm); gap: var(--space-xs); }
.tab-label { white-space: nowrap; max-width: calc(var(--size-xxl) * 2.5); overflow: hidden; text-overflow: ellipsis; }
[part="Chat.unread"] { min-width: var(--size-xs); padding: 0 var(--space-xxs); text-align: center; font-variant-numeric: tabular-nums; }
[part="Chat.log"] { display: flex; flex-direction: column-reverse; width: stretch; flex: 1; min-height: 0; overflow-y: auto; }
.messages { display: flex; flex-direction: column; width: stretch; margin: 0; padding: var(--space-xs); list-style: none; gap: var(--space-xs); }
[part="Chat.message"] { display: flex; flex-direction: column; width: stretch; gap: var(--space-xxs); padding: var(--space-xs) var(--space-sm); }
[part="Chat.message"][data-kind=own] { align-items: flex-end; }
.head { display: flex; width: stretch; align-items: center; gap: var(--space-sm); }
[part="Chat.message"][data-kind=own] .head { justify-content: flex-end; }
[part="Chat.sender"] { min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
[part="Chat.time"] { flex: none; white-space: nowrap; font-variant-numeric: tabular-nums; }
.mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
[part="Chat.text"] { margin: 0; max-width: stretch; text-align: start; overflow-wrap: anywhere; }
[part="Chat.message"][data-kind=own] [part="Chat.text"] { text-align: end; }
.empty { margin: 0; padding: var(--space-lg); text-align: center; }
[part="Chat.notice"] { margin: 0; flex: none; }
[part="Chat.compose"] { display: flex; width: stretch; gap: var(--space-xs); align-items: stretch; flex: none; }
#Draft { width: stretch; min-width: 0; min-height: var(--size-touch); }
#Send { flex: none; min-height: var(--size-touch); min-width: var(--size-touch); gap: var(--space-xs); }
.send-mark { width: var(--size-xs); height: var(--size-xs); }
.send-mark:dir(rtl) { transform: scaleX(-1); }
@container (width < 400px) { .send-label { display: none; } }
</style>
