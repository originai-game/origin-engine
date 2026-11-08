//! Keyed extent index, independent of collection values and row construction.
//! As in TanStack Virtual, unbuilt items use an estimate and measurements
//! belong to persistent keys. A grid row occupies its tallest cell's extent.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
pub(crate) enum Order<'a> {
    Untracked(&'a [String]),
    Retained { keys: &'a [String], revision: u64 },
}

impl<'a> Order<'a> {
    pub fn keys(self) -> &'a [String] {
        match self {
            Self::Untracked(keys) | Self::Retained { keys, .. } => keys,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Virtualizer {
    pub keys: Vec<String>,
    #[serde(skip)]
    order_revision: Option<u64>,
    measured: BTreeMap<String, f32>,
    invalidated: std::collections::BTreeSet<String>,
    starts: Vec<f32>,
    pub columns: usize,
    estimate: f32,
    gap: f32,
    width: f32,
    pub viewport: f32,
    pub grid: bool,
}

/// Escape path delimiters without collapsing distinct collection keys.
pub(crate) fn item_id(key: &str, grid: bool) -> String {
    let key = key
        .replace('%', "%25")
        .replace('/', "%2F")
        .replace('#', "%23");
    format!("{}#{key}", if grid { "Cell" } else { "Row" })
}

impl Virtualizer {
    pub fn valid_snapshot(&self) -> bool {
        self.columns > 0
            && self.starts.len() == self.keys.len().div_ceil(self.columns) + 1
            && self.starts.first() == Some(&0.0)
            && self.starts.windows(2).all(|w| w[1] > w[0])
            && self.estimate > 0.0
            && self.gap >= 0.0
            && self.viewport >= 0.0
            && self.measured.values().all(|height| *height > 0.0)
            && self
                .keys
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == self.keys.len()
    }

    pub fn invalidate(&mut self, keys: &[String]) {
        self.invalidated.extend(keys.iter().cloned());
    }

    pub fn page(&self, index: usize, down: bool) -> usize {
        let y = self.offset(index) + if down { self.viewport } else { -self.viewport };
        (self.row_at(y.max(0.0)) * self.columns).min(self.keys.len().saturating_sub(1))
    }

    fn row_at(&self, offset: f32) -> usize {
        self.starts
            .partition_point(|&x| x <= offset)
            .saturating_sub(1)
            .min(self.starts.len().saturating_sub(2))
    }

    pub fn offset(&self, index: usize) -> f32 {
        if index >= self.keys.len() {
            return self.starts.last().copied().unwrap_or(0.0);
        }
        self.starts
            .get(index / self.columns.max(1))
            .copied()
            .unwrap_or(0.0)
    }

    pub fn total(&self) -> f32 {
        (self.starts.last().copied().unwrap_or(0.0) - self.gap).max(0.0)
    }

    /// Apply order and measurement changes together, preserving the item at
    /// the viewport's leading edge and its intra-row offset. If it was removed,
    /// retain its next surviving neighbour, else its preceding neighbour.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        order: Order<'_>,
        measurements: &[(String, f32)],
        columns: usize,
        estimate: f32,
        gap: f32,
        width: f32,
        viewport: f32,
        scroll: f32,
    ) -> f32 {
        let keys = order.keys();
        let index = self.row_at(scroll) * self.columns.max(1);
        let inside = scroll - self.offset(index);
        let changed_order = match order {
            Order::Retained { revision, .. } if self.order_revision == Some(revision) => false,
            _ => self.keys != keys,
        };
        self.order_revision = match order {
            Order::Retained { revision, .. } => Some(revision),
            Order::Untracked(_) => None,
        };
        let changed_width = self.width != width;
        let mut changed = changed_order
            || changed_width
            || !self.invalidated.is_empty()
            || self.columns != columns
            || self.estimate != estimate
            || self.gap != gap;
        let present = changed_order.then(|| keys.iter().collect::<std::collections::BTreeSet<_>>());
        let contains = |k: &String| present.as_ref().is_none_or(|set| set.contains(k));
        let surviving = if changed_order {
            self.keys
                .get(index)
                .filter(|key| contains(key))
                .or_else(|| {
                    self.keys
                        .iter()
                        .skip(index)
                        .chain(self.keys.iter().take(index).rev())
                        .find(|key| contains(key))
                })
                .and_then(|key| keys.iter().position(|candidate| candidate == key))
        } else {
            Some(index)
        };
        if changed_width {
            self.measured.clear();
        } else {
            for (key, height) in measurements {
                if !self.invalidated.contains(key)
                    && height.is_finite()
                    && *height > 0.0
                    && self.measured.get(key) != Some(height)
                {
                    self.measured.insert(key.clone(), *height);
                    changed = true;
                }
            }
        }
        for key in std::mem::take(&mut self.invalidated) {
            self.measured.remove(&key);
        }
        if changed_order {
            self.measured.retain(|k, _| contains(k));
            self.keys = keys.to_vec();
        }
        self.columns = columns.max(1);
        self.estimate = estimate;
        self.gap = gap;
        self.width = width;
        self.viewport = viewport;
        if changed {
            self.starts.clear();
            self.starts.push(0.0);
            for row in self.keys.chunks(self.columns) {
                let height = row
                    .iter()
                    .map(|k| self.measured.get(k).copied().unwrap_or(estimate))
                    .fold(0.0_f32, f32::max);
                self.starts
                    .push(self.starts.last().copied().unwrap() + height + gap);
            }
        }
        let offset = surviving.map_or(scroll, |i| self.offset(i) + inside);
        offset.clamp(0.0, (self.total() - viewport).max(0.0))
    }

    pub fn range(&self, scroll: f32) -> crate::ListRange {
        let first = self.row_at(scroll).saturating_sub(1) * self.columns;
        let last = (self.row_at(scroll + self.viewport) + 2) * self.columns;
        crate::ListRange {
            first: first.min(self.keys.len()) as u32,
            last: last.min(self.keys.len()) as u32,
        }
    }

    pub fn reveal(&self, index: usize, scroll: f32) -> f32 {
        let top = self.offset(index);
        let bottom = self.offset((index / self.columns + 1) * self.columns) - self.gap;
        let target = if top < scroll {
            top
        } else if bottom > scroll + self.viewport {
            (bottom - self.viewport).min(top)
        } else {
            scroll
        };
        target.clamp(0.0, (self.total() - self.viewport).max(0.0))
    }
}
