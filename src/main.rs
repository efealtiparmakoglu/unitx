use std::process::exit;

use unitx::parser::{evaluate, ParseError};

const USAGE: &str = r#"unitx v1.0.0 — dimensional analysis engine with derived units

USAGE
  unitx eval "<expression>"       evaluate a quantity expression
  unitx convert <value> <from> <to>
  unitx (-h | --help | -V | --version)

EXPRESSIONS
  3.5 m/s * 2 s        multiplication and division track dimensions
  90 km/h              compound units with implicit products
  (2 m)^3              powers, also on units (m^2, s^-1)
  1 N + 1 kg*m/s^2     matching dimensions add; clashing ones error

UNITS: m km cm mm mi ft in | kg g mg t lb oz | s ms us ns min h day |
A mA | K | mol | cd | N J kJ MJ W kW MW Pa kPa bar atm Hz kHz MHz GHz
C V rad deg

Exit codes: 0 ok, 1 evaluation failure, 2 usage error."#;

fn fail_usage(message: &str) -> ! {
    eprintln!("unitx: {}", message);
    eprintln!("unitx: run 'unitx --help' for usage");
    exit(2);
}

fn fail_eval(message: String) -> ! {
    eprintln!("unitx: {}", message);
    exit(1);
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.iter().any(|a| a == "-h" || a == "--help") {
        println!("{}", USAGE);
        return;
    }
    if argv.iter().any(|a| a == "-V" || a == "--version") {
        println!("unitx 1.0.0");
        return;
    }
    match argv.first().map(|s| s.as_str()) {
        Some("eval") => {
            let expression = argv.get(1).cloned().unwrap_or_else(|| {
                fail_usage("eval requires a quoted expression");
            });
            match evaluate(&expression) {
                Ok(q) => println!("{:.6} {}", q.si, q.dim.describe()),
                Err(e) => fail_eval(match e {
                    ParseError::Dimension(m) => m,
                    other => format!("{}", other),
                }),
            }
        }
        Some("convert") => {
            if argv.len() != 4 {
                fail_usage("convert requires exactly: <value> <from> <to>");
            }
            let value: f64 = argv[1]
                .parse()
                .unwrap_or_else(|_| fail_usage(&format!("\"{}\" is not a number", argv[1])));
            let (quantity, converted) = match unitx::parser::convert(value, &argv[2], &argv[3]) {
                Ok(result) => result,
                Err(e) => fail_eval(format!("{}", e)),
            };
            println!("{} {} = {:.6} {}", value, argv[2], converted, argv[3]);
            println!("(SI: {:.6} {})", quantity.si, quantity.dim.describe());
        }
        Some(other) => fail_usage(&format!("unknown command \"{}\"", other)),
        None => fail_usage("missing command"),
    }
}
