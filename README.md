# unitx

A dimensional-analysis engine: unit conversion **plus** dimension
checking, so `m/s + s` is a type error with a human message. Zero
crates.

```text
$ unitx eval "3.5 m/s * 2 s"
7.000000 m

$ unitx eval "1 m + 1 s"
unitx: cannot add m to s: dimensions m and s do not match

$ unitx convert 1 atm bar
1 atm = 1.013250 bar
(SI: 101325.000000 m^-1·kg^1·s^-2)
```

## How it works

- **Dimensions** are exponent vectors over the seven SI bases
  (m, kg, s, A, K, mol, cd). Multiplying quantities adds exponents,
  dividing subtracts, powers scale them — implemented as proper
  `Mul`/`Div` trait impls, so the compiler checks what the units check.
- **Derived units** are defined through their base expressions: the
  parser resolves `N` as `kg·m/s²`, `J`, `W`, `Pa`, `bar`, `atm`, `Hz`,
  `C`, `V` and more, so `1 N + 1 kg*m/s^2` is accepted (= 2 N) while
  mismatched arithmetic names both offending dimensions.
- **Expression parser** — recursive descent with `+ - * /`, unary
  minus, parentheses, `^` powers on quantities and units, compound
  units (`90 km/h`), and implicit products (`3 m s`).
- **Errors** say what happened: unknown units are quoted, syntax errors
  carry a byte offset, conversion failures explain the dimension clash.

## Usage

```
unitx eval "<expression>"
unitx convert <value> <from> <to>
```

Note: temperatures are absolute (K); offset units like °C are
deliberately out of scope because offsets break multiplicative
analysis.

## Development

```console
$ cargo test     # 13 tests: dimension tracking, derived units, errors, conversions
$ cargo clippy --all-targets -- -D warnings
```

## License

MIT — see [LICENSE](LICENSE).
