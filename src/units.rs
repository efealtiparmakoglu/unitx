//! Dimensions, the unit table and quantity arithmetic.

/// Exponents over the seven SI base dimensions:
/// m, kg, s, A, K, mol, cd.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Dimension {
    pub exp: [i8; 7],
}

impl Dimension {
    pub const DIMENSIONLESS: Dimension = Dimension { exp: [0; 7] };

    pub fn base(name: &str) -> Option<Dimension> {
        let index = match name {
            "m" => 0,
            "kg" => 1,
            "s" => 2,
            "A" => 3,
            "K" => 4,
            "mol" => 5,
            "cd" => 6,
            _ => return None,
        };
        let mut exp = [0i8; 7];
        exp[index] = 1;
        Some(Dimension { exp })
    }

    pub fn pow(self, n: i8) -> Dimension {
        let mut exp = [0i8; 7];
        for (e, base) in exp.iter_mut().zip(self.exp.iter()) {
            *e = base * n;
        }
        Dimension { exp }
    }

    /// Human name for the seven base-composed dimensions; falls back to
    /// the raw exponent vector for anything exotic.
    pub fn describe(&self) -> String {
        match self.exp {
            [0, 0, 0, 0, 0, 0, 0] => return "dimensionless".into(),
            [1, 0, 0, 0, 0, 0, 0] => return "m".into(),
            [0, 1, 0, 0, 0, 0, 0] => return "kg".into(),
            [0, 0, 1, 0, 0, 0, 0] => return "s".into(),
            [1, 0, -1, 0, 0, 0, 0] => return "m/s".into(),
            [1, 1, -2, 0, 0, 0, 0] => return "N (kg·m/s²)".into(),
            [2, 1, -2, 0, 0, 0, 0] => return "J (kg·m²/s²)".into(),
            [2, 1, -3, 0, 0, 0, 0] => return "W (kg·m²/s³)".into(),
            [-1, 0, -2, 0, 0, 0, 0] => return "Hz²? (1/s²)".into(),
            _ => {}
        }
        let mut parts: Vec<String> = Vec::new();
        let names = ["m", "kg", "s", "A", "K", "mol", "cd"];
        for (name, &exp) in names.iter().zip(self.exp.iter()) {
            if exp != 0 {
                parts.push(format!("{}^{}", name, exp));
            }
        }
        parts.join("·")
    }
}

// Exponent vectors ADD under unit multiplication — the arithmetic inside
// is intentionally additive, which clippy's Mul lint cannot know.
#[allow(clippy::suspicious_arithmetic_impl)]
impl std::ops::Mul for Dimension {
    type Output = Dimension;
    fn mul(self, other: Dimension) -> Dimension {
        let mut exp = [0i8; 7];
        for (e, (a, b)) in exp.iter_mut().zip(self.exp.iter().zip(other.exp.iter())) {
            *e = a + b;
        }
        Dimension { exp }
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl std::ops::Div for Dimension {
    type Output = Dimension;
    fn div(self, other: Dimension) -> Dimension {
        let mut exp = [0i8; 7];
        for (e, (a, b)) in exp.iter_mut().zip(self.exp.iter().zip(other.exp.iter())) {
            *e = a - b;
        }
        Dimension { exp }
    }
}

/// A unit: multiplier to SI plus its dimension.
#[derive(Clone, Copy, Debug)]
pub struct Unit {
    pub factor: f64,
    pub dim: Dimension,
}

fn unit(name: &'static str, factor: f64, dim: Dimension) -> (&'static str, Unit) {
    (name, Unit { factor, dim })
}

/// Built-in unit table: bases, common prefixes and derived units defined
/// through their base expression (parsed lazily by `lookup` callers).
pub fn builtin_units() -> Vec<(&'static str, Unit)> {
    let m = Dimension::base("m").unwrap();
    let kg = Dimension::base("kg").unwrap();
    let s = Dimension::base("s").unwrap();
    let amp = Dimension::base("A").unwrap();
    let kelvin = Dimension::base("K").unwrap();
    let mol = Dimension::base("mol").unwrap();
    let cd = Dimension::base("cd").unwrap();
    let none = Dimension::DIMENSIONLESS;

    let speed = m / s;
    let accel = speed / s;
    let force = kg * accel;
    let energy = force * m;
    let power = energy / s;
    let pressure = force / (m * m);
    let freq = none / s;
    let charge = amp * s;
    let voltage = energy / charge;

    vec![
        unit("m", 1.0, m),
        unit("km", 1000.0, m),
        unit("dm", 0.1, m),
        unit("cm", 0.01, m),
        unit("mm", 0.001, m),
        unit("mi", 1609.344, m),
        unit("ft", 0.3048, m),
        unit("in", 0.0254, m),
        unit("kg", 1.0, kg),
        unit("g", 0.001, kg),
        unit("mg", 1e-6, kg),
        unit("t", 1000.0, kg),
        unit("lb", 0.45359237, kg),
        unit("oz", 0.028349523125, kg),
        unit("s", 1.0, s),
        unit("ms", 0.001, s),
        unit("us", 1e-6, s),
        unit("ns", 1e-9, s),
        unit("min", 60.0, s),
        unit("h", 3600.0, s),
        unit("day", 86400.0, s),
        unit("A", 1.0, amp),
        unit("mA", 0.001, amp),
        unit("K", 1.0, kelvin),
        unit("mol", 1.0, mol),
        unit("cd", 1.0, cd),
        unit("N", 1.0, force),
        unit("J", 1.0, energy),
        unit("kJ", 1000.0, energy),
        unit("MJ", 1e6, energy),
        unit("W", 1.0, power),
        unit("kW", 1000.0, power),
        unit("MW", 1e6, power),
        unit("Pa", 1.0, pressure),
        unit("kPa", 1000.0, pressure),
        unit("bar", 100000.0, pressure),
        unit("atm", 101325.0, pressure),
        unit("Hz", 1.0, freq),
        unit("kHz", 1000.0, freq),
        unit("MHz", 1e6, freq),
        unit("GHz", 1e9, freq),
        unit("C", 1.0, charge),
        unit("V", 1.0, voltage),
        unit("rad", 1.0, none),
        unit("deg", std::f64::consts::PI / 180.0, none),
    ]
}

/// A quantity: value in SI units plus its dimension.
#[derive(Clone, Copy, Debug)]
pub struct Quantity {
    pub si: f64,
    pub dim: Dimension,
}

impl Quantity {
    pub fn new(value: f64, unit: Unit) -> Quantity {
        Quantity { si: value * unit.factor, dim: unit.dim }
    }

    /// Checked because dimension mismatches are user errors worth naming.
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, other: Quantity) -> Result<Quantity, String> {
        if self.dim != other.dim {
            return Err(format!(
                "cannot add {} to {}: dimensions {} and {} do not match",
                self.dim.describe(),
                other.dim.describe(),
                self.dim.describe(),
                other.dim.describe()
            ));
        }
        Ok(Quantity { si: self.si + other.si, dim: self.dim })
    }

    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, other: Quantity) -> Result<Quantity, String> {
        if self.dim != other.dim {
            return Err(format!(
                "cannot subtract {} from {}: dimensions {} and {} do not match",
                other.dim.describe(),
                self.dim.describe(),
                other.dim.describe(),
                self.dim.describe()
            ));
        }
        Ok(Quantity { si: self.si - other.si, dim: self.dim })
    }

    pub fn div_checked(self, other: Quantity) -> Result<Quantity, String> {
        if other.si == 0.0 {
            return Err("division by zero".into());
        }
        Ok(Quantity { si: self.si / other.si, dim: self.dim / other.dim })
    }

    pub fn pow(self, n: i8) -> Quantity {
        Quantity { si: self.si.powi(n as i32), dim: self.dim.pow(n) }
    }
}

impl std::ops::Mul for Quantity {
    type Output = Quantity;
    fn mul(self, other: Quantity) -> Quantity {
        Quantity { si: self.si * other.si, dim: self.dim * other.dim }
    }
}
