// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
// The IDL's list constants (`List.String`): each const's value is its members,
// whitespace-separated in one string, as a readonly array. Emitted apart from
// exports.ts, whose column factories import the runtime: provider and host
// modules read these without it.

// platform

/** The core methods that are not service methods: the store and its save slots, the leaderboards, telemetry and the catalog's head — what any host answers from its own storage. The provider's CORE_METHODS is these plus SERVICE_METHODS; capabilities is these, the services and the extensions. */
export const CORE_ONLY_METHODS = ["store.get", "store.set", "store.list", "leaderboard.submit", "leaderboard.top", "leaderboard.around", "telemetry", "catalog.head"] as const;

/** The service methods: purchases, ads, achievements, the player, friends, invites and share (docs/platform-asks.md #19). They need the platform's services, so a host without one answers it `unavailable`. */
export const SERVICE_METHODS = ["purchase.products", "purchase.buy", "purchase.entitlements", "purchase.consume", "ad.ready", "ad.show", "achievement.unlock", "achievement.progress", "achievement.list", "player.get", "friends.list", "invite.send", "invite.list", "share"] as const;

/** Methods one host defines: `point` (Origin Point) and `ai.mark` are OriginGame's, not core methods (docs/platform-asks.md). */
export const HOST_EXTENSIONS = ["point", "ai.mark"] as const;

/** Every error code an answer may carry (PlatformError.code); a call may be retried later for RETRYABLE_CODES. */
export const ERROR_CODES = ["offline", "timeout", "invalid", "unauthorized", "not_found", "conflict", "too_large", "rate_limited", "unavailable", "failed", "cancelled", "no_fill"] as const;

/** The error codes whose call may succeed later (`retryable: true`). */
export const RETRYABLE_CODES = ["offline", "timeout", "unavailable", "rate_limited", "no_fill"] as const;
