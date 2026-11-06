// A test's own time limit for a slower machine: OG_TEST_TIME_SCALE
// multiplies it (a slow runner sets 3).
// The limit stays what it is everywhere else.
export const slow = (ms: number): number => ms * Math.max(1, Number(process.env.OG_TEST_TIME_SCALE) || 1);
