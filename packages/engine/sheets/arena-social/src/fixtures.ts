// Fixture data mirrors the three component documents; the native test checks
// this copy so the review cannot silently show an easier state than the API.
export const FIXTURES: Record<string, Record<string, Record<string, unknown>>> = {
  ready: {
    default: { title: "", detail: "Ranked · Foundry", timeText: "12s", players: [{ id: "rhea", name: "Rhea", state: "ready" }, { id: "kai", name: "Kai", state: "pending" }, { id: "ana", name: "Ana", state: "declined" }], response: "", canRespond: true, closed: false, loading: false },
    selected: { players: [{ id: "rhea", name: "Rhea", state: "ready" }, { id: "kai", name: "Kai", state: "ready" }], response: "ready" },
    disabled: { canRespond: false, detail: "Only joined players can respond" },
    closed: { closed: true, detail: "One player declined. Returning to queue.", response: "declined" },
    loading: { loading: true },
    empty: { players: [], detail: "Waiting for participants" },
    overflow: { title: "Championship Series · Grand Final ready check", detail: "A response is needed from every player before the tournament can continue", players: [{ id: "long", name: "Maximilian Vandenberghe-Okonkwo", state: "pending" }, { id: "offline", name: "Abernathy-Whitcombe", state: "offline" }, { id: "rhea", name: "Rhea", state: "ready" }, { id: "kai", name: "Kai", state: "declined" }] },
  },
  vote: {
    default: { title: "", detail: "Choose the next map. You can change your vote.", timeText: "18s", options: [{ id: "foundry", name: "Foundry", icon: "fire", detail: "Close quarters · 2 v 2", votes: 3, disabled: false, reason: "" }, { id: "grove", name: "Moonlit Grove", icon: "leaf", detail: "Open lanes · 2 v 2", votes: 1, disabled: false, reason: "" }, { id: "citadel", name: "Citadel", icon: "crown", detail: "Tournament map", votes: 0, disabled: true, reason: "Not installed" }], selected: "foundry", winner: "", canVote: true, closed: false, loading: false },
    closed: { closed: true, winner: "grove", detail: "Moonlit Grove won the vote.", options: [{ id: "foundry", name: "Foundry", icon: "fire", detail: "Close quarters · 2 v 2", votes: 3, disabled: false, reason: "" }, { id: "grove", name: "Moonlit Grove", icon: "leaf", detail: "Open lanes · 2 v 2", votes: 5, disabled: false, reason: "" }, { id: "citadel", name: "Citadel", icon: "crown", detail: "Tournament map", votes: 0, disabled: true, reason: "Not installed" }] },
    disabled: { canVote: false, detail: "Spectators cannot vote", selected: "" },
    loading: { loading: true },
    empty: { options: [], selected: "", detail: "Waiting for the host to choose maps" },
    overflow: { title: "Championship Series · Deciding map selection", options: [{ id: "isles", name: "The Shattered Isles of the Unforgotten Ancients", icon: "map", detail: "An extended description that must remain readable without pushing a vote count off the phone", votes: 1234567, disabled: false, reason: "" }, { id: "grove", name: "Moonlit Grove", icon: "leaf", detail: "Open lanes", votes: 99, disabled: false, reason: "" }], selected: "isles" },
  },
  spectator: {
    default: { title: "", detail: "Ranked · Round 3 · Broadcast delay 10s", targets: [{ id: "rhea", name: "Rhea", team: "Blue", detail: "Alive", disabled: false }, { id: "kai", name: "Kai", team: "Red", detail: "Respawning · 4s", disabled: false }, { id: "ana", name: "Ana", team: "Blue", detail: "Disconnected", disabled: true }], selected: "kai", freeCamera: false, canFree: true, canCycle: true, canLeave: true, loading: false },
    free: { freeCamera: true },
    disabled: { canFree: false, canCycle: false, canLeave: false, detail: "Camera controls locked during replay transition" },
    loading: { loading: true },
    empty: { targets: [], selected: "", detail: "Waiting for players to join" },
    overflow: { title: "Championship Series · Grand Final spectator view", targets: [{ id: "long", name: "Maximilian Vandenberghe-Okonkwo", team: "The Northern Champions", detail: "Respawning at the eastern forward base in 12 seconds", disabled: false }, { id: "rhea", name: "Rhea", team: "Blue", detail: "Alive", disabled: false }, { id: "kai", name: "Kai", team: "Red", detail: "Alive", disabled: false }], selected: "long" },
  },
};
