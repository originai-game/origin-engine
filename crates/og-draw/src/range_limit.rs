/// How bright the presented frame may be over SDR white: CSS Color HDR 1's
/// `dynamic-range-limit` as its computed value (§3.3), the weights of
/// `standard`, `constrained` and `no-limit`, which sum to 1. The display
/// pass ends the frame at [`RangeLimit::peak`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangeLimit {
    pub standard: f32,
    pub constrained: f32,
    pub no_limit: f32,
}

impl Default for RangeLimit {
    /// `no-limit`, the property's initial value.
    fn default() -> Self {
        RangeLimit {
            standard: 0.0,
            constrained: 0.0,
            no_limit: 1.0,
        }
    }
}

impl RangeLimit {
    /// `constrained`'s headroom in stops over SDR white: one, twice its
    /// luminance (§3.1 leaves "somewhat greater" to the user agent).
    pub const CONSTRAINED_STOPS: f32 = 1.0;

    /// The brightest the frame shows, as a multiple of SDR white, on a
    /// display whose headroom is `headroom` (the same multiple): each
    /// keyword's headroom in stops (`standard` none, `constrained`
    /// [`Self::CONSTRAINED_STOPS`], `no-limit` the display's), mixed by its
    /// weight in stops (§3.2), never past the display's.
    pub fn peak(self, headroom: f32) -> f32 {
        let display = headroom.max(1.0).log2();
        let stops =
            self.constrained * display.min(Self::CONSTRAINED_STOPS) + self.no_limit * display;
        stops.min(display).exp2()
    }
}
