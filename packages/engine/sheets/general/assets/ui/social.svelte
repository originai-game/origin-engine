<meta name="component" content="GeneralSocialPage">

<!-- Desktop: the guild chat beside the friends panel. A phone: the chat
     over the friends. -->
<div id="Page" class="general-ground">
  <div id="Chat"><Chat channels={$chatChannels} channel={$chatChannel} messages={$chatMessages} bind:draft={$chatDraft} maxLength={200} canSend={true} notice={$chatNotice} /></div>
  <div id="Friends"><Friends title={$blank} requests={$friendsRequests} friends={$friendsList} selected={$friendsSelected} /></div>
</div>

<style>
#Page { display: grid; width: stretch; height: stretch; grid-template-columns: minmax(0, 1.5fr) minmax(0, 1fr); column-gap: var(--space-lg); padding: var(--space-lg); }
#Chat, #Friends { height: stretch; min-height: 0; }
@media (width < 600px) {
  #Page { display: flex; flex-direction: column; gap: var(--space-md); padding: var(--space-sm); }
  #Chat { width: stretch; height: 52%; flex: none; }
  #Friends { width: stretch; flex: 1; min-height: 0; }
}
</style>
