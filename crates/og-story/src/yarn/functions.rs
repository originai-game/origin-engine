//! Standard-library value operations, ported from Yarn StandardLibrary.cs (MIT).
//! Stateful functions are handled by the story, never a process-global RNG.
use super::types::{Function, Types};
use crate::expr::{Expr, Scope, Val};
use std::collections::{BTreeMap, BTreeSet};

/// Validate the closed call graph before cook. Bodies have no access to a host,
/// story variables, RNG, clocks or quests, including through another function.
pub fn validate(functions: &mut BTreeMap<String, Function>) -> Result<(), String> {
    fn calls(e: &Expr, names: &mut Vec<String>) -> Result<(), String> {
        match e {
            Expr::Invoke(name, args) => {
                if matches!(
                    name.as_str(),
                    "random"
                        | "random_range"
                        | "random_range_float"
                        | "dice"
                        | "visited"
                        | "visited_count"
                        | "has_any_content"
                ) {
                    return Err(format!("pure function cannot call stateful '{name}'"));
                }
                names.push(name.clone());
                for arg in args {
                    calls(arg, names)?;
                }
            }
            Expr::Once(_) | Expr::Call(_) => {
                return Err("pure function cannot read quest or story state".into());
            }
            Expr::Yarn(e) | Expr::Not(e) | Expr::Neg(e) => calls(e, names)?,
            Expr::Bin(_, a, b) => {
                calls(a, names)?;
                calls(b, names)?;
            }
            _ => {}
        }
        Ok(())
    }
    let signatures: BTreeMap<_, _> = functions
        .iter()
        .map(|(n, f)| (n.clone(), f.signature.clone()))
        .collect();
    let mut graph = BTreeMap::new();
    for (name, function) in functions.iter_mut() {
        if super::types::builtin(name).is_some() {
            return Err(format!("cannot replace builtin function '{name}'"));
        }
        if function.parameters.len() != function.signature.parameters.len()
            || function.signature.variadic.is_some()
        {
            return Err(format!("'{name}' needs a fixed typed parameter list"));
        }
        let variables: BTreeMap<_, _> = function
            .parameters
            .iter()
            .cloned()
            .zip(function.signature.parameters.iter().cloned())
            .collect();
        if variables.len() != function.parameters.len() {
            return Err(format!("'{name}' repeats a parameter"));
        }
        let mut reads = Vec::new();
        function.body.vars(&mut reads);
        if let Some(var) = reads.iter().find(|v| !variables.contains_key(*v)) {
            return Err(format!("'{name}' cannot read non-parameter ${var}"));
        }
        let mut types = Types {
            variables,
            functions: signatures.clone(),
            ..Default::default()
        };
        types.infer(&mut function.body, Some(&function.signature.returns))?;
        types.require_resolved(&function.body)?;
        let mut dependencies = Vec::new();
        calls(&function.body, &mut dependencies)?;
        graph.insert(name.clone(), dependencies);
    }
    fn visit(
        name: &str,
        graph: &BTreeMap<String, Vec<String>>,
        path: &mut BTreeSet<String>,
    ) -> Result<(), String> {
        if !path.insert(name.into()) {
            return Err(format!("recursive pure function '{name}'"));
        }
        if let Some(calls) = graph.get(name) {
            for call in calls {
                visit(call, graph, path)?;
            }
        }
        path.remove(name);
        Ok(())
    }
    for name in graph.keys() {
        visit(name, &graph, &mut BTreeSet::new())?;
    }
    Ok(())
}

pub fn invoke(
    functions: &BTreeMap<String, Function>,
    name: &str,
    args: &[Val],
) -> Result<Val, String> {
    if let Some(value) = pure(name, args)? {
        return Ok(value);
    }
    let f = functions
        .get(name)
        .ok_or_else(|| format!("function '{name}' is not registered"))?;
    if args.len() != f.parameters.len() {
        return Err(format!("'{name}' argument count mismatch"));
    }
    struct Parameters<'a> {
        functions: &'a BTreeMap<String, Function>,
        values: BTreeMap<String, Val>,
    }
    impl Scope for Parameters<'_> {
        fn var(&self, name: &str) -> Option<Val> {
            self.values.get(name).cloned()
        }
        fn seen(&self, _: &str) -> bool {
            false
        }
        fn quest_status(&self, _: &str) -> &'static str {
            "none"
        }
        fn quest_stage(&self, _: &str) -> String {
            String::new()
        }
        fn quest_count(&self, _: &str, _: &str) -> f64 {
            0.0
        }
        fn invoke(&self, name: &str, args: &[Val]) -> Result<Val, String> {
            invoke(self.functions, name, args)
        }
    }
    f.body.eval(&Parameters {
        functions,
        values: f
            .parameters
            .iter()
            .cloned()
            .zip(args.iter().cloned())
            .collect(),
    })
}

pub fn string(value: &Val) -> String {
    match value {
        Val::Bool(true) => "True".into(),
        Val::Bool(false) => "False".into(),
        Val::Num(n) => general(*n as f32, None, 'E'),
        Val::Str(s) => s.clone(),
    }
}

fn number(value: &Val) -> Result<f32, String> {
    match value {
        Val::Num(n) => Ok(*n as f32),
        Val::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        Val::Str(s) => s
            .trim()
            .parse()
            .map_err(|_| format!("'{s}' is not a number")),
    }
}

/// None means an engine-state function or a registered function, not zero.
pub fn pure(name: &str, args: &[Val]) -> Result<Option<Val>, String> {
    let signature = super::types::builtin(name);
    if let Some(signature) = &signature
        && args.len() != signature.parameters.len()
    {
        return Err(format!(
            "{name} expects {} arguments",
            signature.parameters.len()
        ));
    }
    let n = |i: usize| {
        args.get(i)
            .ok_or_else(|| format!("{name} missing argument {i}"))
            .and_then(number)
    };
    let numeric = |value: f32| Some(Val::Num(f64::from(value)));
    Ok(match name {
        "min" => numeric(n(0)?.min(n(1)?)),
        "max" => numeric(n(0)?.max(n(1)?)),
        "round" => numeric(n(0)?.round_ties_even()),
        "round_places" => {
            let places = n(1)?.round_ties_even();
            if !(0.0..=15.0).contains(&places) {
                return Err("round_places needs 0 through 15 decimal places".into());
            }
            let mut factor = 1.0_f64;
            for _ in 0..places as u32 {
                factor *= 10.0;
            }
            numeric(((f64::from(n(0)?) * factor).round_ties_even() / factor) as f32)
        }
        "floor" => numeric(n(0)?.floor()),
        "ceil" => numeric(n(0)?.ceil()),
        "int" => numeric(n(0)?.trunc()),
        "decimal" => numeric(n(0)? - n(0)?.trunc()),
        "inc" => {
            let value = n(0)?;
            numeric(if value.fract() == 0.0 {
                value + 1.0
            } else {
                value.ceil()
            })
        }
        "dec" => {
            let value = n(0)?;
            numeric(if value.fract() == 0.0 {
                value - 1.0
            } else {
                value.floor()
            })
        }
        "number" => numeric(n(0)?),
        "string" | "format_invariant" => Some(Val::Str(string(&args[0]))),
        "bool" => Some(Val::Bool(match &args[0] {
            Val::Bool(b) => *b,
            Val::Num(n) => *n != 0.0,
            Val::Str(s) if s.trim().eq_ignore_ascii_case("true") => true,
            Val::Str(s) if s.trim().eq_ignore_ascii_case("false") => false,
            Val::Str(s) => return Err(format!("'{s}' is not a boolean")),
        })),
        "format" => Some(Val::Str(format(&string(&args[0]), &args[1])?)),
        _ => None,
    })
}

/// v3.2.2 passes the typed argument to .NET composite formatting. The later
/// development branch's stringification would incorrectly ignore numeric specs.
fn format(template: &str, argument: &Val) -> Result<String, String> {
    let mut chars = template.chars().peekable();
    let mut output = String::new();
    while let Some(c) = chars.next() {
        match c {
            '{' | '}' if chars.peek() == Some(&c) => {
                chars.next();
                output.push(c);
            }
            '{' => {
                let mut field = String::new();
                let mut closed = false;
                for c in chars.by_ref() {
                    if c == '}' {
                        closed = true;
                        break;
                    }
                    if c == '{' {
                        return Err("nested brace in format field".into());
                    }
                    field.push(c);
                }
                if !closed {
                    return Err("unclosed format field".into());
                }
                let (field, spec) = field.split_once(':').unwrap_or((&field, ""));
                let argument = match argument {
                    Val::Num(n) if !spec.is_empty() => numeric_format(*n as f32, spec)?,
                    value => string(value),
                };
                let (index, alignment) = field
                    .split_once(',')
                    .map_or((field, None), |(i, a)| (i, Some(a)));
                if index.trim() != "0" {
                    return Err("format has only argument {0}".into());
                }
                let alignment = alignment
                    .map(|a| a.trim().parse::<i32>())
                    .transpose()
                    .map_err(|_| "invalid format alignment")?
                    .unwrap_or(0);
                let width = alignment.unsigned_abs() as usize;
                let padding = width.saturating_sub(argument.encode_utf16().count());
                if padding > 1_000_000 {
                    return Err("format alignment exceeds the story text limit".into());
                }
                if alignment > 0 {
                    output.extend(std::iter::repeat_n(' ', padding));
                }
                output.push_str(&argument);
                if alignment < 0 {
                    output.extend(std::iter::repeat_n(' ', padding));
                }
            }
            '}' => return Err("unescaped closing brace in format".into()),
            c => output.push(c),
        }
    }
    Ok(output)
}

fn numeric_format(value: f32, spec: &str) -> Result<String, String> {
    if !value.is_finite() {
        return Ok(special(value).unwrap().into());
    }
    if !spec.as_bytes()[0].is_ascii_alphabetic()
        || !spec.as_bytes()[1..].iter().all(u8::is_ascii_digit)
    {
        return custom_format(value, spec);
    }
    let (kind, digits) = spec.split_at(1);
    let precision = if digits.is_empty() {
        None
    } else {
        Some(
            digits
                .parse::<usize>()
                .map_err(|_| format!("numeric format '{spec}' is not implemented"))?,
        )
    };
    if precision.is_some_and(|p| p > 999_999_999) {
        return Err("numeric format precision exceeds 999999999".into());
    }
    let fixed_precision = precision.unwrap_or(2);
    match kind {
        "f" | "F" => Ok(format!("{value:.fixed_precision$}")),
        "n" | "N" => Ok(group(&format!("{value:.fixed_precision$}"))),
        "c" | "C" => {
            let amount = group(&format!("{:.fixed_precision$}", value.abs()));
            Ok(if value.is_sign_negative() {
                format!("(¤{amount})")
            } else {
                format!("¤{amount}")
            })
        }
        "p" | "P" => Ok(format!(
            "{} %",
            group(&format!("{:.fixed_precision$}", f64::from(value) * 100.0))
        )),
        "e" | "E" => {
            let precision = precision.unwrap_or(6);
            let scientific = format!("{value:.precision$e}");
            let (mantissa, exponent) = scientific.split_once('e').unwrap();
            Ok(format!(
                "{mantissa}{kind}{}",
                exponent_text(exponent.parse().unwrap(), 3)
            ))
        }
        "g" | "G" => Ok(general(
            value,
            precision.filter(|p| *p != 0),
            if kind == "g" { 'e' } else { 'E' },
        )),
        "r" | "R" => Ok(general(value, None, if kind == "r" { 'e' } else { 'E' })),
        _ => Err(format!("invalid floating-point numeric format '{spec}'")),
    }
}

fn special(value: f32) -> Option<&'static str> {
    if value.is_nan() {
        Some("NaN")
    } else if value == f32::INFINITY {
        Some("Infinity")
    } else if value == f32::NEG_INFINITY {
        Some("-Infinity")
    } else {
        None
    }
}

fn exponent_text(exponent: i32, width: usize) -> String {
    format!(
        "{}{:0width$}",
        if exponent < 0 { '-' } else { '+' },
        exponent.unsigned_abs()
    )
}

fn general(value: f32, precision: Option<usize>, letter: char) -> String {
    if let Some(text) = special(value) {
        return text.into();
    }
    let scientific = if let Some(precision) = precision {
        let places = precision.saturating_sub(1);
        format!("{:.places$e}", value.abs())
    } else {
        format!("{:e}", value.abs())
    };
    let (mantissa, exponent) = scientific.split_once('e').unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    let digits = mantissa.replace('.', "");
    let digits = digits.trim_end_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    let sign = if value.is_sign_negative() { "-" } else { "" };
    if exponent < -4 || exponent >= precision.unwrap_or(9) as i32 {
        let fraction = if digits.len() > 1 {
            format!(".{}", &digits[1..])
        } else {
            String::new()
        };
        format!(
            "{sign}{}{fraction}{letter}{}",
            &digits[..1],
            exponent_text(exponent, 2)
        )
    } else {
        let point = exponent + 1;
        if point <= 0 {
            format!("{sign}0.{}{digits}", "0".repeat((-point) as usize))
        } else if point as usize >= digits.len() {
            format!(
                "{sign}{digits}{}",
                "0".repeat(point as usize - digits.len())
            )
        } else {
            format!(
                "{sign}{}.{}",
                &digits[..point as usize],
                &digits[point as usize..]
            )
        }
    }
}

fn group(fixed: &str) -> String {
    let (integer, fraction) = fixed.split_once('.').unwrap_or((fixed, ""));
    let mut grouped = String::new();
    for (i, c) in integer.chars().enumerate() {
        if i > 0
            && c.is_ascii_digit()
            && (integer.len() - i).is_multiple_of(3)
            && integer.as_bytes()[i - 1].is_ascii_digit()
        {
            grouped.push(',');
        }
        grouped.push(c);
    }
    if fixed.contains('.') {
        grouped.push('.');
        grouped.push_str(fraction);
    }
    grouped
}

#[derive(Clone, PartialEq)]
enum NumericToken {
    Digit(bool),
    Point,
    Comma,
    Percent,
    Permille,
    Exponent(char, bool, usize),
    Literal(char),
}

fn custom_format(value: f32, spec: &str) -> Result<String, String> {
    use NumericToken::*;
    let mut sections = vec![Vec::new()];
    let mut chars = spec.chars().peekable();
    while let Some(c) = chars.next() {
        let tokens = sections.last_mut().unwrap();
        match c {
            ';' => sections.push(Vec::new()),
            '\\' => {
                if let Some(c) = chars.next() {
                    tokens.push(Literal(c));
                }
            }
            '\'' | '"' => {
                for quoted in chars.by_ref() {
                    if quoted == c {
                        break;
                    }
                    tokens.push(Literal(quoted));
                }
            }
            '0' => tokens.push(Digit(true)),
            '#' => tokens.push(Digit(false)),
            '.' => tokens.push(Point),
            ',' => tokens.push(Comma),
            '%' => tokens.push(Percent),
            '‰' => tokens.push(Permille),
            'e' | 'E' => {
                let mut lookahead = chars.clone();
                let positive = lookahead.peek() == Some(&'+');
                if matches!(lookahead.peek(), Some('+' | '-')) {
                    lookahead.next();
                }
                let mut zeroes = 0;
                while lookahead.peek() == Some(&'0') {
                    zeroes += 1;
                    lookahead.next();
                }
                if zeroes > 0 {
                    chars = lookahead;
                    tokens.push(Exponent(c, positive, zeroes));
                } else {
                    tokens.push(Literal(c));
                }
            }
            c => tokens.push(Literal(c)),
        }
    }
    let section = if value == 0.0 && sections.get(2).is_some_and(|s| !s.is_empty()) {
        2
    } else if value < 0.0 && sections.get(1).is_some_and(|s| !s.is_empty()) {
        1
    } else {
        0
    };
    custom_section(value, &sections, section)
}

fn custom_section(
    value: f32,
    sections: &[Vec<NumericToken>],
    section: usize,
) -> Result<String, String> {
    use NumericToken::*;
    let tokens = &sections[section];
    let point = tokens
        .iter()
        .position(|t| *t == Point)
        .unwrap_or(tokens.len());
    let exponent_at = tokens
        .iter()
        .position(|t| matches!(t, Exponent(..)))
        .unwrap_or(tokens.len());
    let integral: Vec<_> = tokens
        .iter()
        .enumerate()
        .filter_map(|(i, t)| (i < point.min(exponent_at) && matches!(t, Digit(_))).then_some(i))
        .collect();
    let fractional: Vec<_> = tokens
        .iter()
        .enumerate()
        .filter_map(|(i, t)| (i > point && i < exponent_at && matches!(t, Digit(_))).then_some(i))
        .collect();
    let mut shift = tokens
        .iter()
        .map(|t| match t {
            Percent => 2,
            Permille => 3,
            _ => 0,
        })
        .sum::<i32>();
    let mut grouping = false;
    if let Some(last) = integral.last() {
        for (i, token) in tokens.iter().enumerate().take(point.min(exponent_at)) {
            if *token == Comma {
                if i > *last {
                    shift -= 3;
                } else {
                    grouping = true;
                }
            }
        }
    }
    // .NET's custom Single formats first generate seven significant decimal
    // digits, then round the requested decimal position away from zero.
    let scientific = format!("{:.6e}", value.abs());
    let (mantissa, exponent) = scientific.split_once('e').unwrap();
    let mut digits: Vec<u8> = mantissa
        .bytes()
        .filter(|b| *b != b'.')
        .map(|b| b - b'0')
        .collect();
    let mut decimal = exponent.parse::<i32>().unwrap() + 1 + shift;
    let mut output_exponent = 0;
    if exponent_at != tokens.len() {
        output_exponent = if value == 0.0 {
            0
        } else {
            decimal - integral.len() as i32
        };
        decimal = integral.len() as i32;
    }
    let keep = decimal + fractional.len() as i32;
    if keep >= 0 && (keep as usize) < digits.len() {
        let round = digits[keep as usize] >= 5;
        digits.truncate(keep as usize);
        if round {
            let mut cursor = digits.len();
            while cursor > 0 && digits[cursor - 1] == 9 {
                cursor -= 1;
                digits[cursor] = 0;
            }
            if cursor > 0 {
                digits[cursor - 1] += 1;
            } else {
                digits.insert(0, 1);
                decimal += 1;
            }
        }
    } else if keep < 0 {
        digits.clear();
    }
    let zero = digits.iter().all(|d| *d == 0);
    if zero && section != 2 && sections.get(2).is_some_and(|s| !s.is_empty()) {
        return custom_section(0.0, sections, 2);
    }
    if zero && section == 1 {
        return custom_section(-0.0, sections, 0);
    }
    if exponent_at != tokens.len() && decimal > integral.len() as i32 {
        output_exponent += decimal - integral.len() as i32;
        decimal = integral.len() as i32;
    }
    let digit = |place: i32| -> char {
        if place < 0 {
            '0'
        } else {
            char::from(b'0' + digits.get(place as usize).copied().unwrap_or(0))
        }
    };
    let mut integer = if zero || decimal <= 0 {
        String::new()
    } else {
        (0..decimal).map(digit).collect::<String>()
    };
    let minimum = if exponent_at != tokens.len() {
        integral.len()
    } else {
        integral
            .iter()
            .position(|i| tokens[*i] == Digit(true))
            .map_or(0, |i| integral.len() - i)
    };
    if integer.len() < minimum {
        integer = format!("{}{integer}", "0".repeat(minimum - integer.len()));
    }
    let mut fraction: String = (0..fractional.len())
        .map(|i| digit(decimal + i as i32))
        .collect();
    let min_fraction = fractional
        .iter()
        .rposition(|i| tokens[*i] == Digit(true))
        .map_or(0, |i| i + 1);
    while fraction.len() > min_fraction && fraction.ends_with('0') {
        fraction.pop();
    }
    let mut output = String::new();
    let mut int_index = 0;
    let mut frac_index = 0;
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Digit(_) if i < point.min(exponent_at) => {
                let remaining_slots = integral.len() - int_index;
                let count = integer.len().saturating_sub(remaining_slots - 1);
                for (j, c) in integer[..count].chars().enumerate() {
                    output.push(c);
                    if grouping
                        && integer.len() - j > 1
                        && (integer.len() - j - 1).is_multiple_of(3)
                    {
                        output.push(',');
                    }
                }
                integer.drain(..count);
                int_index += 1;
            }
            Digit(_) if i > point && i < exponent_at => {
                if let Some(c) = fraction.as_bytes().get(frac_index) {
                    output.push(char::from(*c));
                }
                frac_index += 1;
            }
            Point if i == point && !fraction.is_empty() => output.push('.'),
            Exponent(letter, positive, width) => {
                output.push(*letter);
                if *positive || output_exponent < 0 {
                    output.push_str(&exponent_text(output_exponent, *width));
                } else {
                    output.push_str(&format!("{:0width$}", output_exponent, width = *width));
                }
            }
            Percent => output.push('%'),
            Permille => output.push('‰'),
            Literal(c) => output.push(*c),
            _ => {}
        }
    }
    if section == 0 && value.is_sign_negative() && !output.is_empty() {
        output.insert(0, '-');
    }
    Ok(output)
}
