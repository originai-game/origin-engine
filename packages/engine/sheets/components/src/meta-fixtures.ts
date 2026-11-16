// Review facts are owned by the sample game, not by the documents.
const rewards = [
  { id: "gold", tier: "1", premium: false, name: "Gold", icon: "coin", amount: "500", detail: "", state: "claimed" },
  { id: "keys", tier: "4", premium: false, name: "Chest keys", icon: "key", amount: "3", detail: "", state: "available" },
  { id: "blade", tier: "4", premium: true, name: "Ember Blade", icon: "sword", amount: "1", detail: "Legendary weapon skin", state: "available" },
  { id: "gems", tier: "5", premium: false, name: "Gems", icon: "gem", amount: "100", detail: "", state: "locked" },
];
const pass = { title: "", season: "Season of Embers", refresh: "Ends in 12 days", progress: .4, progressText: "Level 4 · 240 / 500 XP", premiumUnlocked: false, upgradeLabel: "View premium pass", rewards, loading: false, disabled: false, error: "" };
const reveal = { title: "Expedition rewards", body: "Open the chest to see what your party recovered.", state: "sealed", rewards: [], canSkip: true, continueLabel: "", disabled: false, error: "" };
const prize = [{ id: "blade", name: "Ember Blade of the Ninefold Gate", icon: "sword", amount: "1", rarity: "legendary", fresh: true }, { id: "gold", name: "Gold", icon: "coin", amount: "1,250", rarity: "common", fresh: false }];
const messages = [
  { id: "welcome", subject: "Welcome to the expedition", sender: "Expedition Guild", date: "16 Nov", preview: "Supplies for your first journey.", body: "Welcome, adventurer. These supplies will help you reach the northern camp.\n\nGood luck on your first expedition.", unread: true, hasAttachments: true, claimable: true, claimed: false, expired: false, canDelete: true },
  { id: "notice", subject: "Scheduled maintenance", sender: "Game team", date: "15 Nov", preview: "Service resumes at 09:00 UTC.", body: "The world will be unavailable during maintenance. All progress is safe.", unread: false, hasAttachments: false, claimable: false, claimed: false, expired: false, canDelete: true },
];
const inbox = { title: "", summary: "1 unread · 2 / 50", selected: "welcome", messages, attachments: [{ id: "gold", message: "welcome", name: "Gold", icon: "coin", amount: "500" }, { id: "potions", message: "welcome", name: "Health potions", icon: "potion", amount: "3" }], loading: false, busy: false, error: "" };
const entries = [
  { id: "patch", title: "Expedition update 1.4", date: "16 Nov 2026", kind: "Patch notes", summary: "New destinations, equipment balance and accessibility improvements.", body: "New destinations\nExplore the northern coast with your party.\n\nAccessibility\nController navigation now remembers the last focused item.\n\nBalance\nHeavy weapons recover faster after a missed attack.", image: 0, unread: true, pinned: true, actionLabel: "View full release notes" },
  { id: "event", title: "Double rewards weekend", date: "20–22 Nov", kind: "Event", summary: "Earn extra expedition rewards all weekend.", body: "All expedition rewards are doubled during the event.", image: 0, unread: false, pinned: false, actionLabel: "" },
];
const news = { title: "", selected: "patch", entries, loading: false, disabled: false, error: "" };
export const META_FIXTURES: Record<string, Record<string, Record<string, unknown>>> = {
  passTrack: {
    default: pass,
    empty: { ...pass, rewards: [] }, loading: { ...pass, loading: true }, error: { ...pass, error: "The season service is unavailable. Your progress is saved." },
    premium: { ...pass, premiumUnlocked: true, progress: 1, progressText: "Season complete", rewards: [{ ...rewards[2], id: "skin", tier: "50", name: "Watcher of the Ninefold Gate", detail: "An exclusive outfit for completing this season’s premium track." }] },
    disabled: { ...pass, disabled: true },
  },
  reveal: {
    default: reveal, loading: { ...reveal, state: "loading" }, revealing: { ...reveal, state: "revealing" },
    revealed: { ...reveal, title: "Expedition complete", body: "", state: "revealed", rewards: prize },
    empty: { ...reveal, state: "revealed" }, error: { ...reveal, error: "Your rewards are saved. Reconnect to view them." },
    disabled: { ...reveal, state: "revealed", rewards: prize, disabled: true },
  },
  inbox: {
    default: inbox, empty: { ...inbox, summary: "", selected: "", messages: [], attachments: [] },
    loading: { ...inbox, loading: true }, error: { ...inbox, error: "Messages could not be loaded. Reconnect and try again." },
    expired: { ...inbox, messages: [{ ...messages[0], claimable: false, expired: true }] },
    claimed: { ...inbox, messages: [{ ...messages[0], unread: false, claimable: false, claimed: true }] },
    busy: { ...inbox, busy: true }, unselected: { ...inbox, selected: "" },
    overflow: { ...inbox, messages: [{ ...messages[0], subject: "Rewards for the northern expedition and the return of the Watcher of the Ninefold Gate", sender: "The northern territories expedition coordination team", body: "Congratulations on completing the northern expedition.\n\nYour party’s progress and equipment are safe. These supplies can be claimed from any character on your account.\n\nThank you for playing." }] },
  },
  news: {
    default: news, empty: { ...news, entries: [], selected: "" }, loading: { ...news, loading: true }, error: { ...news, error: "Updates are temporarily unavailable. Try again when connected." },
    collapsed: { ...news, selected: "" }, disabled: { ...news, disabled: true },
    overflow: { ...news, entries: [{ ...entries[0], title: "The northern expedition and the return of the Watcher of the Ninefold Gate", date: "16 November 2026 · 09:00 UTC", kind: "Community event and balance notes", actionLabel: "Read the complete expedition update" }] },
  },
};
export const META_NAMES: Record<string, string> = { passTrack: "PassTrack", reveal: "Reveal", inbox: "Inbox", news: "News" };
