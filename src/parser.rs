//! Recursive-descent parser for unit expressions such as
//! `3.5 m/s * 2 s` or `(2 km/h)^2`.

use crate::units::{builtin_units, Quantity, Unit};

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    /// Unknown identifier (unit name).
    UnknownUnit { name: String },
    /// Syntax problem, with the byte offset in the input.
    Syntax { at: usize, message: &'static str },
    /// Arithmetic-level error surfaced while folding (e.g. dimension clash).
    Dimension(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnknownUnit { name } => {
                write!(f, "unknown unit \"{name}\"")
            }
            ParseError::Syntax { at, message } => {
                write!(f, "syntax error at byte {at}: {message}")
            }
            ParseError::Dimension(message) => write!(f, "{message}"),
        }
    }
}

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
    units: Vec<(&'static str, Unit)>,
}

/// Evaluate a full unit expression to one quantity.
pub fn evaluate(text: &str) -> Result<Quantity, ParseError> {
    let mut parser = Parser { input: text.as_bytes(), pos: 0, units: builtin_units() };
    let value = parser.parse_expr()?;
    parser.skip_ws();
    if parser.pos != parser.input.len() {
        return Err(ParseError::Syntax {
            at: parser.pos,
            message: "unexpected trailing input",
        });
    }
    Ok(value)
}

/// Look up one unit name and convert `value` into SI.
pub fn convert(value: f64, from: &str, to: &str) -> Result<(Quantity, f64), ParseError> {
    let parser = Parser { input: b"", pos: 0, units: builtin_units() };
    let from_unit = parser.lookup(from)?;
    let to_unit = parser.lookup(to)?;
    if from_unit.dim != to_unit.dim {
        let quantity = Quantity::new(value, from_unit);
        return Err(ParseError::Dimension(format!(
            "cannot convert {} to {}: dimensions {} and {} do not match",
            value,
            to,
            quantity.dim.describe(),
            to_unit.dim.describe()
        )));
    }
    let quantity = Quantity::new(value, from_unit);
    Ok((quantity, value * from_unit.factor / to_unit.factor))
}

impl<'a> Parser<'a> {
    fn skip_ws(&mut self) {
        while self.pos < self.input.len() && (self.input[self.pos] as char).is_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn lookup(&self, name: &str) -> Result<Unit, ParseError> {
        self.units
            .iter()
            .find(|(candidate, _)| *candidate == name)
            .map(|(_, unit)| *unit)
            .ok_or_else(|| ParseError::UnknownUnit { name: name.to_string() })
    }

    fn parse_expr(&mut self) -> Result<Quantity, ParseError> {
        let mut left = self.parse_term()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some(b'+') => {
                    self.pos += 1;
                    let right = self.parse_term()?;
                    left = left.add(right).map_err(ParseError::Dimension)?;
                }
                Some(b'-') => {
                    self.pos += 1;
                    let right = self.parse_term()?;
                    left = left.sub(right).map_err(ParseError::Dimension)?;
                }
                _ => return Ok(left),
            }
        }
    }

    fn parse_term(&mut self) -> Result<Quantity, ParseError> {
        let mut left = self.parse_factor()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some(b'*') => {
                    self.pos += 1;
                    let right = self.parse_factor()?;
                    left = left * right;
                }
                Some(b'/') => {
                    self.pos += 1;
                    let right = self.parse_factor()?;
                    left = left.div_checked(right).map_err(ParseError::Dimension)?;
                }
                // implicit multiplication by a unit ("2 m" already handled
                // in parse_factor; space-separated units like "3 m s" count too)
                _ => return Ok(left),
            }
        }
    }

    fn parse_factor(&mut self) -> Result<Quantity, ParseError> {
        self.skip_ws();
        match self.peek() {
            Some(b'(') => {
                self.pos += 1;
                let value = self.parse_expr()?;
                self.skip_ws();
                if self.peek() != Some(b')') {
                    return Err(ParseError::Syntax {
                        at: self.pos,
                        message: "expected closing parenthesis",
                    });
                }
                self.pos += 1;
                self.parse_power(value)
            }
            Some(c) if c == b'-' || c == b'+' => {
                self.pos += 1;
                let value = self.parse_factor()?;
                Ok(if c == b'-' {
                    Quantity { si: -value.si, dim: value.dim }
                } else {
                    value
                })
            }
            Some(c) if (c as char).is_ascii_digit() || c == b'.' => {
                let number = self.parse_number()?;
                let quantity = self.parse_unit_part(number)?;
                self.parse_power(quantity)
            }
            Some(_) => {
                // a bare unit expression like "km" (implicit value 1)
                let name_start = self.pos;
                if self.consume_identifier() {
                    let name = std::str::from_utf8(&self.input[name_start..self.pos])
                        .map_err(|_| ParseError::Syntax { at: name_start, message: "bad UTF-8" })?
                        .to_string();
                    let unit = self.lookup(&name)?;
                    return self.parse_power(Quantity::new(1.0, unit));
                }
                Err(ParseError::Syntax {
                    at: self.pos,
                    message: "expected a number, unit or '('",
                })
            }
            None => Err(ParseError::Syntax {
                at: self.pos,
                message: "unexpected end of input",
            }),
        }
    }

    /// After a number: an optional unit expression (product of unit powers,
    /// possibly divided), e.g. `5 m/s^2`, `90 km/h`.
    fn parse_unit_part(&mut self, number: f64) -> Result<Quantity, ParseError> {
        self.skip_ws();
        if !self.starts_identifier() {
            return Ok(Quantity::new(number, dimensionless_unit()));
        }
        let numerator = self.parse_unit_power()?;
        let mut quantity = Quantity::new(number, numerator);
        self.skip_ws();
        if self.peek() == Some(b'/') {
            self.pos += 1;
            // the denominator is a full factor: `5 m / 0 s` and `90 km/h`
            // both land here
            let denominator = self.parse_factor()?;
            quantity = quantity.div_checked(denominator).map_err(ParseError::Dimension)?;
        }
        // "3 m s" style implicit products
        loop {
            self.skip_ws();
            let here = self.pos;
            if !self.starts_identifier() {
                self.pos = here;
                break;
            }
            let next = self.parse_unit_power()?;
            quantity = quantity * Quantity::new(1.0, next);
        }
        Ok(quantity)
    }

    fn parse_unit_power(&mut self) -> Result<Unit, ParseError> {
        let start = self.pos;
        if !self.consume_identifier() {
            return Err(ParseError::Syntax {
                at: start,
                message: "expected a unit name",
            });
        }
        let name = std::str::from_utf8(&self.input[start..self.pos])
            .map_err(|_| ParseError::Syntax { at: start, message: "bad UTF-8" })?
            .to_string();
        let unit = self.lookup(&name)?;
        self.skip_ws();
        if self.peek() == Some(b'^') {
            self.pos += 1;
            self.skip_ws();
            let mut exponent: i8 = 1;
            if self.peek() == Some(b'-') {
                self.pos += 1;
                exponent = -1;
            }
            let mut magnitude: i8 = 0;
            while let Some(c) = self.peek() {
                if (c as char).is_ascii_digit() {
                    magnitude = magnitude * 10 + (c - b'0') as i8;
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if magnitude == 0 {
                return Err(ParseError::Syntax { at: self.pos, message: "exponent must be 1..=99" });
            }
            let power = exponent * magnitude;
            return Ok(Unit {
                factor: unit.factor.powi(power as i32),
                dim: unit.dim.pow(power),
            });
        }
        Ok(unit)
    }

    fn parse_power(&mut self, value: Quantity) -> Result<Quantity, ParseError> {
        self.skip_ws();
        if self.peek() == Some(b'^') {
            self.pos += 1;
            self.skip_ws();
            let mut sign = 1i8;
            if self.peek() == Some(b'-') {
                self.pos += 1;
                sign = -1;
            }
            let mut exponent: i8 = 0;
            while let Some(c) = self.peek() {
                if (c as char).is_ascii_digit() {
                    exponent = exponent * 10 + (c - b'0') as i8;
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if exponent == 0 {
                return Err(ParseError::Syntax { at: self.pos, message: "exponent must be 1..=99" });
            }
            return Ok(value.pow(sign * exponent));
        }
        Ok(value)
    }

    fn starts_identifier(&self) -> bool {
        matches!(self.peek(), Some(c) if (c as char).is_ascii_alphabetic() || c == b'_' )
    }

    fn consume_identifier(&mut self) -> bool {
        if !self.starts_identifier() {
            return false;
        }
        while matches!(self.peek(), Some(c) if (c as char).is_ascii_alphanumeric() || c == b'_') {
            self.pos += 1;
        }
        true
    }

    fn parse_number(&mut self) -> Result<f64, ParseError> {
        let start = self.pos;
        while matches!(self.peek(), Some(c) if (c as char).is_ascii_digit() || c == b'.') {
            self.pos += 1;
        }
        let text = std::str::from_utf8(&self.input[start..self.pos])
            .map_err(|_| ParseError::Syntax { at: start, message: "bad UTF-8" })?;
        text.parse::<f64>()
            .map_err(|_| ParseError::Syntax { at: start, message: "invalid number" })
    }
}

fn dimensionless_unit() -> Unit {
    Unit { factor: 1.0, dim: crate::units::Dimension::DIMENSIONLESS }
}
