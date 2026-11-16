// Public-prop fixtures for the new base documents; no game or reward rules.
const empty = { title: "No saved builds", body: "Save your equipment to switch builds between matches.", hint: "Your current equipment will not change.", icon: "bag", actionLabel: "Save this build", disabled: false };
const error = { title: "Could not load your builds", body: "Your saved builds are safe. Check your connection and try again.", code: "NET-204", retryLabel: "", canRetry: true, busy: false, dismissLabel: "Go back" };
const row = { title: "Ember Blade", subtitle: "Legendary sword · Equipped by Mira", icon: "sword", image: 0, value: "1,250", badge: "New", selected: true, disabled: false, loading: false };
const steps = { title: "Prepare your expedition", column: false, steps: [
  { id: "party", label: "Party", detail: "3 players ready", number: "1", state: "done", disabled: false },
  { id: "route", label: "Route", detail: "Choose a destination", number: "2", state: "current", disabled: false },
  { id: "launch", label: "Launch", detail: "Waiting for a route", number: "3", state: "upcoming", disabled: true },
] };

export const BASE_FIXTURES: Record<string, Record<string, Record<string, unknown>>> = {
  emptyState: {
    default: empty,
    minimal: { ...empty, title: "", body: "", hint: "", actionLabel: "" },
    disabled: { ...empty, body: "Finish the tutorial before saving a build.", disabled: true },
    overflow: { ...empty, title: "No discoveries in the northern reaches of the Sunken Kingdom", body: "Explore the coast and investigate the old watchtowers to add discoveries to this collection.", hint: "Progress is shared across every character on your account.", actionLabel: "Return to the world map and choose a destination" },
  },
  errorState: {
    default: error,
    busy: { ...error, title: "", body: "Connecting to the server…", code: "", busy: true },
    terminal: { ...error, title: "This replay is no longer available", body: "It was recorded on an older game version.", code: "", canRetry: false, dismissLabel: "Return to replays" },
    overflow: { ...error, title: "We could not retrieve the shared expedition report", body: "The host may have left the session. Your completed objectives are saved, and you can return to the lobby without losing rewards.", code: "EXPEDITION-REPORT-UNAVAILABLE", dismissLabel: "Return to the expedition lobby" },
  },
  listItem: {
    default: row,
    disabled: { ...row, title: "Northern expedition", subtitle: "Reach level 12 to unlock this destination.", icon: "lock", value: "", badge: "", selected: false, disabled: true },
    loading: { ...row, title: "Shared build", subtitle: "Retrieving equipment…", icon: "bag", value: "", badge: "", selected: false, loading: true },
    overflow: { ...row, title: "The Ember Blade of the Last Watcher of the Ninefold Gate", subtitle: "A two-handed legendary sword with an unusually long description that must remain readable on a phone.", value: "1,250,000 gold", badge: "Recommended" },
  },
  steps: {
    default: steps,
    empty: { ...steps, title: "", steps: [] },
    error: { ...steps, title: "", column: true, steps: [{ id: "gear", label: "Equipment", detail: "", number: "1", state: "done", disabled: false }, { id: "network", label: "Connection", detail: "Reconnect to continue", number: "2", state: "error", disabled: false }, { id: "ready", label: "Ready", detail: "", number: "3", state: "upcoming", disabled: true }] },
    overflow: { ...steps, title: "An expedition across the northern territories", steps: [{ id: "one", label: "Choose your character and starting equipment", detail: "Every choice can be changed later in the lobby.", number: "1", state: "done", disabled: false }, { id: "two", label: "Invite the rest of your expedition party", detail: "Waiting for players to accept their invitations.", number: "2", state: "current", disabled: false }, { id: "three", label: "Confirm the expedition destination", detail: "", number: "3", state: "upcoming", disabled: true }] },
  },
};

export const BASE_NAMES: Record<string, string> = { emptyState: "EmptyState", errorState: "ErrorState", listItem: "ListItem", steps: "Steps" };
