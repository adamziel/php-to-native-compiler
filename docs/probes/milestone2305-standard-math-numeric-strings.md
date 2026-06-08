# Milestone 2305 Standard Math Numeric String Probes

These probes record the PHP-src PHPT rows used to bound the current
standard-math numeric-string slice. They are proof notes only; the executable
coverage is in `compiler/tests/standard_math_residuals.rs` and
`tests/fixtures/milestone2305/standard_math_numeric_strings.php`.

## PHP-src PHPT rows consulted

- `ext/standard/tests/math/is_finite_basic.phpt`
  (`https://github.com/php/php-src/blob/master/ext/standard/tests/math/is_finite_basic.phpt`)
  covers integer, float, full numeric-string, boolean, infinite, and NaN inputs
  for `is_finite()`.
- `ext/standard/tests/math/sqrt_variation.phpt`
  (`https://github.com/php/php-src/blob/master/ext/standard/tests/math/sqrt_variation.phpt`)
  covers scalar numeric strings and booleans for `sqrt()`.
- `ext/standard/tests/math/abs_basic.phpt`
  (`https://github.com/php/php-src/blob/master/ext/standard/tests/math/abs_basic.phpt`)
  covers integer-shaped strings, float-shaped strings, `null`, and booleans
  for `abs()`.

## Focused local controls

The local fixture intentionally avoids broad libm parity. It proves only the
shared deterministic classifier boundary:

```text
bool(false)
float(INF)
39|-3|4.5
123abc:is_finite(): Argument #1 ($num) must be of type float, string given
INF:is_finite(): Argument #1 ($num) must be of type float, string given
NAN:is_finite(): Argument #1 ($num) must be of type float, string given
```

That means full numeric strings, including exponent overflow, are accepted by
the selected helpers, while leading-numeric junk and symbolic string `INF`/`NAN`
remain catchable `TypeError` cases. Object/resource numeric casts, exact
diagnostic parity for every argument label, broad platform-dependent libm
results, and native lowering stay unsupported.
