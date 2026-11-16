export const FIXTURES: Record<string, Record<string, unknown>> = {
  default: { name: "Brenna Vale", handle: "@brenna", portrait: 0, level: 12, status: "online", rank: "Gold II · Season 4", bio: "Support main. Here for the team.", stats: [{ id: "wins", label: "Wins", value: "128", detail: "This season" }, { id: "matches", label: "Matches", value: "204", detail: "Ranked" }, { id: "best", label: "Best streak", value: "9", detail: "Consecutive wins" }], actions: [{ id: "friend", label: "Add friend", disabled: false, reason: "", pending: false }, { id: "invite", label: "Invite", disabled: true, reason: "Player is in a match", pending: false }], privateStats: false, loading: false, error: "", empty: false },
  private: { privateStats: true, status: "offline" },
  pending: { actions: [{ id: "friend", label: "Add friend", disabled: false, reason: "", pending: true }] },
  loading: { loading: true },
  error: { error: "Profile could not be loaded. Check your connection." },
  empty: { empty: true },
  "no-stats": { stats: [], actions: [] },
  overflow: { name: "Maximilian Vandenberghe-Okonkwo", handle: "@the-unforgotten-champion-of-the-eastern-isles", rank: "Grandmaster · Northern Championship Series", bio: "A player biography that takes several lines on a phone and never hides the player actions or pushes statistics beyond the screen width.", stats: [{ id: "damage", label: "Highest accumulated damage in a single championship", value: "123,456,789,012", detail: "All seasons combined" }, { id: "wins", label: "Wins", value: "12,345", detail: "Across all regions" }], actions: [{ id: "friend", label: "Send a companion request", disabled: false, reason: "", pending: false }, { id: "invite", label: "Invite to the championship lobby", disabled: true, reason: "This player does not accept invitations from non-friends", pending: false }] },
};
