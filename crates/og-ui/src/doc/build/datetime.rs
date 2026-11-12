//! HTML's `<input type="date|time|datetime-local|month|week"
//! bind:value={$when}>` (docs/ui.md §2): a pad-friendly picker built of
//! og-ui's number steppers, one per part (year, month, day, week, hour,
//! minute, second), each a focus stop that Left and Right, a press or a
//! held press step; no OS picker. The value is HTML's (`2026-11-08`,
//! `14:30`, `2026-11-08T14:30`, `2026-11`, `2026-W45`), kept within `min`
//! and `max` and on the `step` grid from `min` as HTML's step base is.
//! Pure calendar arithmetic (proleptic Gregorian, ISO weeks): no clock.

use super::*;

/// The five kinds and the parts each shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DateType {
    Date,
    Time,
    DateTime,
    Month,
    Week,
}

impl DateType {
    pub(crate) fn from_name(name: &str) -> Option<DateType> {
        Some(match name {
            "date" => DateType::Date,
            "time" => DateType::Time,
            "datetime-local" => DateType::DateTime,
            "month" => DateType::Month,
            "week" => DateType::Week,
            _ => return None,
        })
    }

    /// HTML's default step and the scale of `step`: seconds for times,
    /// days, months or weeks for the rest.
    fn default_step(self) -> i64 {
        match self {
            DateType::Time | DateType::DateTime => 60,
            _ => 1,
        }
    }
}

/// Days since 1970-01-01 of a civil date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

fn days_in_month(y: i64, m: i64) -> i64 {
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    days_from_civil(ny, nm, 1) - days_from_civil(y, m, 1)
}

/// The Monday of ISO week 1 of `y`.
fn week_one(y: i64) -> i64 {
    let jan4 = days_from_civil(y, 1, 4);
    jan4 - (jan4 + 3).rem_euclid(7)
}

fn weeks_in_year(y: i64) -> i64 {
    (week_one(y + 1) - week_one(y)) / 7
}

/// A value as one number on its type's scale: days, months or weeks since
/// 1970's first, seconds into the day, or seconds since 1970-01-01T00:00.
fn parse(ty: DateType, s: &str) -> Option<i64> {
    let num = |t: &str, n: usize| {
        (t.len() == n && t.bytes().all(|b| b.is_ascii_digit()))
            .then(|| t.parse::<i64>().ok())
            .flatten()
    };
    let date = |t: &str| {
        let mut it = t.split('-');
        let (y, m, d) = (
            num(it.next()?, 4)?,
            num(it.next()?, 2)?,
            num(it.next()?, 2)?,
        );
        (it.next().is_none() && (1..=12).contains(&m) && (1..=days_in_month(y, m)).contains(&d))
            .then(|| days_from_civil(y, m, d))
    };
    let time = |t: &str| {
        let mut it = t.split(':');
        let (h, m) = (num(it.next()?, 2)?, num(it.next()?, 2)?);
        let sec = match it.next() {
            Some(x) => num(x, 2)?,
            None => 0,
        };
        (it.next().is_none() && h < 24 && m < 60 && sec < 60).then_some(h * 3600 + m * 60 + sec)
    };
    match ty {
        DateType::Date => date(s),
        DateType::Time => time(s),
        DateType::DateTime => {
            let (d, t) = s.split_once('T')?;
            Some(date(d)? * 86_400 + time(t)?)
        }
        DateType::Month => {
            let (y, m) = s.split_once('-')?;
            let (y, m) = (num(y, 4)?, num(m, 2)?);
            (1..=12).contains(&m).then_some(y * 12 + m - 1 - 1970 * 12)
        }
        DateType::Week => {
            let (y, w) = s.split_once("-W")?;
            let (y, w) = (num(y, 4)?, num(w, 2)?);
            (1..=weeks_in_year(y))
                .contains(&w)
                .then_some((week_one(y) + 7 * (w - 1) - week_one(1970)) / 7)
        }
    }
}

fn hms(s: i64, seconds: bool) -> String {
    let (h, m, sec) = (s / 3600, s / 60 % 60, s % 60);
    if seconds {
        format!("{h:02}:{m:02}:{sec:02}")
    } else {
        format!("{h:02}:{m:02}")
    }
}

fn format(ty: DateType, v: i64, seconds: bool) -> String {
    let ymd = |d: i64| {
        let (y, m, d) = civil_from_days(d);
        format!("{y:04}-{m:02}-{d:02}")
    };
    match ty {
        DateType::Date => ymd(v),
        DateType::Time => hms(v, seconds),
        DateType::DateTime => format!(
            "{}T{}",
            ymd(v.div_euclid(86_400)),
            hms(v.rem_euclid(86_400), seconds)
        ),
        DateType::Month => format!(
            "{:04}-{:02}",
            (v + 1970 * 12).div_euclid(12),
            (v + 1970 * 12).rem_euclid(12) + 1
        ),
        DateType::Week => {
            let monday = week_one(1970) + 7 * v;
            let (y, ..) = civil_from_days(monday + 3);
            format!("{y:04}-W{:02}", (monday - week_one(y)) / 7 + 1)
        }
    }
}

/// The parts a value splits into: `(id, label, value, min, max)`.
fn parts(ty: DateType, v: i64, seconds: bool) -> Vec<(&'static str, &'static str, i64, i64, i64)> {
    let mut out = Vec::new();
    let day_parts = |out: &mut Vec<_>, d: i64| {
        let (y, m, dd) = civil_from_days(d);
        out.push(("Year", "Year", y, 1, 9999));
        out.push(("Month", "Month", m, 1, 12));
        out.push(("Day", "Day", dd, 1, days_in_month(y, m)));
    };
    let time_parts = |out: &mut Vec<_>, s: i64| {
        out.push(("Hour", "Hour", s / 3600, 0, 23));
        out.push(("Minute", "Minute", s / 60 % 60, 0, 59));
        if seconds {
            out.push(("Second", "Second", s % 60, 0, 59));
        }
    };
    match ty {
        DateType::Date => day_parts(&mut out, v),
        DateType::Time => time_parts(&mut out, v),
        DateType::DateTime => {
            day_parts(&mut out, v.div_euclid(86_400));
            time_parts(&mut out, v.rem_euclid(86_400));
        }
        DateType::Month => {
            let n = v + 1970 * 12;
            out.push(("Year", "Year", n.div_euclid(12), 1, 9999));
            out.push(("Month", "Month", n.rem_euclid(12) + 1, 1, 12));
        }
        DateType::Week => {
            let monday = week_one(1970) + 7 * v;
            let (y, ..) = civil_from_days(monday + 3);
            out.push(("Year", "Year", y, 1, 9999));
            out.push((
                "Week",
                "Week",
                (monday - week_one(y)) / 7 + 1,
                1,
                weeks_in_year(y),
            ));
        }
    }
    out
}

/// The value the parts make: a part past its end carries into the next
/// (the 30th of February is March 1st, minute 60 the next hour); a month
/// or year moved keeps the day in its month (`keep_day`).
fn join(ty: DateType, p: &[i64], keep_day: bool) -> i64 {
    let day = |y: i64, m: i64, d: i64| {
        let (y, m) = (y + (m - 1).div_euclid(12), (m - 1).rem_euclid(12) + 1);
        days_from_civil(
            y,
            m,
            if keep_day {
                d.min(days_in_month(y, m))
            } else {
                d
            },
        )
    };
    let secs = |p: &[i64]| p[0] * 3600 + p[1] * 60 + p.get(2).copied().unwrap_or(0);
    match ty {
        DateType::Date => day(p[0], p[1], p[2]),
        DateType::Time => secs(p).rem_euclid(86_400),
        DateType::DateTime => day(p[0], p[1], p[2]) * 86_400 + secs(&p[3..]),
        DateType::Month => p[0] * 12 + p[1] - 1 - 1970 * 12,
        DateType::Week => {
            let w = if keep_day {
                p[1].min(weeks_in_year(p[0]))
            } else {
                p[1]
            };
            (week_one(p[0]) + 7 * (w - 1) - week_one(1970)) / 7
        }
    }
}

/// `v` within `min` and `max` and on the step grid from `min` (HTML's
/// step base; else 0), rounded to the nearest step.
fn constrain(v: i64, min: Option<i64>, max: Option<i64>, step: i64) -> i64 {
    let base = min.unwrap_or(0);
    let mut v = base + ((v - base) as f64 / step as f64).round() as i64 * step;
    if let Some(hi) = max
        && v > hi
    {
        v -= (v - hi + step - 1) / step * step;
    }
    if let Some(lo) = min
        && v < lo
    {
        v = lo;
    }
    v
}

/// Whether `value` breaks the control's constraints: empty and required,
/// not a valid value of its type, or outside `min`…`max`.
pub(super) fn invalid(r: &Resolved) -> bool {
    let Some(ty) = r.str("dateType").and_then(DateType::from_name) else {
        return false;
    };
    let value = r.str("dateValue").unwrap_or_default();
    if value.is_empty() {
        return r.flag("required");
    }
    let Some(v) = parse(ty, value) else {
        return true;
    };
    r.str("dateMin")
        .and_then(|m| parse(ty, m))
        .is_some_and(|m| v < m)
        || r.str("dateMax")
            .and_then(|m| parse(ty, m))
            .is_some_and(|m| v > m)
}

pub(crate) trait DatetimeUi {
    /// Builds a date or time input; answers its response (`changed` when
    /// it wrote a value back).
    fn datetime_input(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &str,
        r: &Resolved,
    ) -> crate::ui::Response;
}

impl DatetimeUi for Ui {
    fn datetime_input(
        &mut self,
        b: &mut Build<'_>,
        n: &DocNode,
        id: &str,
        r: &Resolved,
    ) -> crate::ui::Response {
        let ty = r
            .str("dateType")
            .and_then(DateType::from_name)
            .unwrap_or(DateType::Date);
        let min = r.str("dateMin").and_then(|m| parse(ty, m));
        let max = r.str("dateMax").and_then(|m| parse(ty, m));
        let step = match r.num("stepBy") {
            Some(s) if s >= 1.0 => s.round() as i64,
            _ => ty.default_step(),
        };
        // Seconds show when the step or the value has them.
        let value = r.str("dateValue").unwrap_or_default();
        let seconds = matches!(ty, DateType::Time | DateType::DateTime)
            && (step % 60 != 0 || value.matches(':').count() == 2);
        // An empty input starts from `min` (else `max`, else the step
        // base): no clock is read, so a capture is the same every run.
        let now = parse(ty, value)
            .or(min)
            .or(max)
            .unwrap_or_else(|| match ty {
                DateType::Date | DateType::DateTime => {
                    days_from_civil(2000, 1, 1) * if ty == DateType::Date { 1 } else { 86_400 }
                }
                DateType::Month => 30 * 12,
                DateType::Week => (week_one(2000) - week_one(1970)) / 7,
                DateType::Time => 0,
            });
        let disabled = r.flag("disabled") || r.flag("readonly");
        let t = self.env.theme;
        self.open(
            id,
            Element {
                direction: Direction::Row,
                width: r.size("width").unwrap_or_default(),
                gap: t.space.sm,
                cross: crate::layout::Cross::Center,
                wrap: true,
                ..Element::default()
            },
        );
        let at = self.tree.parent();
        self.tree.nodes[at].a11y.role = Some("group");
        if let Some(name) = r.str("name") {
            self.tree.nodes[at].a11y.name = Some(name.to_string());
        }
        self.tree.nodes[at].a11y.value = Some(format(ty, now, seconds));
        let mut values: Vec<i64> = Vec::new();
        let mut changed = false;
        let mut keep_day = false;
        // Every part but the year steps without bounds of its own and
        // carries ([`join`]); the whole value keeps to min and max.
        let all = parts(ty, now, seconds);
        let last = all.len() - 1;
        for (k, (part, label, v, lo, hi)) in all.into_iter().enumerate() {
            // The smallest part steps by the step where it divides evenly
            // (a 15-minute step moves the minutes by 15).
            let unit = match (ty, part) {
                (DateType::Time | DateType::DateTime, "Minute") if !seconds => 60,
                (DateType::DateTime, "Day") => 86_400,
                _ => 1,
            };
            let by = if k == last && step % unit == 0 && step / unit <= hi - lo {
                (step / unit).max(1)
            } else {
                1
            };
            let out = self.number_stepper(
                part,
                v as f32,
                &crate::widgets::NumberStepperDesc {
                    min: (part == "Year").then_some(lo as f32),
                    max: (part == "Year").then_some(hi as f32),
                    step: by as f32,
                    label: label.to_string(),
                    format: crate::widgets::ValueFormat {
                        digits: if part == "Year" { 4 } else { 2 },
                        ..Default::default()
                    },
                    disabled,
                    ..Default::default()
                },
            );
            changed |= out.changed;
            keep_day |= out.changed && matches!(part, "Year" | "Month");
            values.push(if out.changed {
                out.value.round() as i64
            } else {
                v
            });
        }
        self.close();
        if changed {
            let next = constrain(join(ty, &values, keep_day), min, max, step);
            self.docs
                .write_back(b, n, "dateValue", json!(format(ty, next, seconds)));
        }
        crate::ui::Response {
            changed,
            ..crate::ui::Response::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Leap days, ISO weeks at year ends, steps from `min` and bounds:
    /// each value checked against a calendar, not against this code.
    #[test]
    fn values_follow_the_calendar_and_html_steps() {
        use DateType::*;
        assert_eq!(parse(Date, "2024-02-30"), None);
        assert_eq!(
            format(Date, parse(Date, "2024-02-29").unwrap() + 1, false),
            "2024-03-01"
        );
        assert_eq!(
            format(Date, join(Date, &[2023, 2, 31], true), false),
            "2023-02-28"
        );
        assert_eq!(
            format(Date, join(Date, &[2023, 12, 32], false), false),
            "2024-01-01"
        );
        assert_eq!(
            format(Date, join(Date, &[2023, 13, 5], true), false),
            "2024-01-05"
        );
        assert_eq!(format(Time, join(Time, &[23, 60], false), false), "00:00");
        // 2020 has 53 ISO weeks; 2021-W01 starts on Monday 2021-01-04.
        assert_eq!(weeks_in_year(2020), 53);
        assert_eq!(weeks_in_year(2021), 52);
        let w = parse(Week, "2020-W53").unwrap();
        assert_eq!(format(Week, w + 1, false), "2021-W01");
        assert_eq!(week_one(2021), days_from_civil(2021, 1, 4));
        // 2026-11-08 is a Sunday in ISO week 45.
        assert_eq!(
            format(
                Week,
                (days_from_civil(2026, 11, 8) - week_one(1970)).div_euclid(7),
                false
            ),
            "2026-W45"
        );
        assert_eq!(
            format(Month, parse(Month, "1999-12").unwrap() + 1, false),
            "2000-01"
        );
        assert_eq!(
            format(
                DateTime,
                parse(DateTime, "2026-12-31T23:59").unwrap() + 60,
                false
            ),
            "2027-01-01T00:00"
        );
        // A 15-minute grid from 09:05, kept under 10:00.
        let (lo, hi) = (parse(Time, "09:05"), parse(Time, "10:00"));
        assert_eq!(
            format(
                Time,
                constrain(parse(Time, "09:27").unwrap(), lo, hi, 900),
                false
            ),
            "09:20"
        );
        assert_eq!(
            format(
                Time,
                constrain(parse(Time, "09:58").unwrap(), lo, hi, 900),
                false
            ),
            "09:50"
        );
        assert_eq!(
            format(
                Time,
                constrain(parse(Time, "08:00").unwrap(), lo, hi, 900),
                false
            ),
            "09:05"
        );
    }
}
