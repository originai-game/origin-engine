// The feedback components (docs/ui.md §2 "Feedback"), merged into og.ui
// by the package entry. Each is declared in crates/og-api/api.ogt (module
// `ui`, `host false`) and built only from og.ui's public members.

import { countBadge } from "./badges.ts";
import { banner, ribbon } from "./banners.ts";
import { celebration, starRating } from "./celebrations.ts";
import { formatNumber, numberCounter } from "./counters.ts";
import { notifications } from "./notifications.ts";
import { progressBar, progressRing, skeletonGroup } from "./progress.ts";
import { subtitles } from "./subtitles.ts";
import { tooltipCard } from "./tooltips.ts";

export const feedback = Object.freeze({
  notifications,
  countBadge,
  tooltipCard,
  progressBar,
  progressRing,
  skeletonGroup,
  formatNumber,
  numberCounter,
  banner,
  ribbon,
  celebration,
  starRating,
  subtitles,
});
