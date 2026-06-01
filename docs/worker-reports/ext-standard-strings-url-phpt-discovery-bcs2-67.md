# ext/standard strings/url PHPT discovery (bcs2-67)

- Compiler source: `1f09a754` (`checkpoint: add parse_url relative rows`).
- PHP source pin: `/home/claude/php-src-phpt` at `f97ff597429a2fe633665a7e02d97c8077f9f90f`.
- Scope: `ext/standard/tests/strings/*.phpt` and `ext/standard/tests/url/*.phpt` only.
- Method: seeded from the latest full-gate status, then verified the 70 smallest stale failures on current master with the PHPT wrapper.
- Focused verification command:

```sh
CARGO_TARGET_DIR=/dev/shm/phpc-target-discovery-bcs2-67 \
  CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  cargo build -p phpc --bin phpc

cd /home/claude/php-src-phpt
PHPC_BIN=/dev/shm/phpc-target-discovery-bcs2-67/debug/phpc \
  TEST_PHP_EXECUTABLE=/home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper \
  TEST_PHP_ARGS= TMPDIR=/tmp/bcs2-67-phpt TEMP=/tmp/bcs2-67-phpt TMP=/tmp/bcs2-67-phpt \
  php run-tests.php -q -p /home/claude/supervised-php-compiler/tools/phpc-phpt-wrapper \
  $(cat /tmp/bcs2-67-small-fail-seed.txt)
```

Focused PHPT result: `70` selected, `67` failed, `3` passed. The passed rows were the now-fixed metaphone rows `bug44242.phpt`, `bug47443.phpt`, and `bug48709.phpt`.

## Ranked candidates

Ranked by current verified PHPT file size among the smallest remaining failures, with rows retained where the observed delta is plausibly bounded to one existing runtime/interpreter builtin, helper, parser surface, or metadata table.

| # | PHPT path | Size | Observed `phpc` vs PHP delta | Likely files/functions | Risk |
|---:|---|---:|---|---|---|
| 1 | `ext/standard/tests/strings/crypt_des_error.phpt` | 114 B | PHP: `crypt('foo', '*0')` returns `string(2) "*1"`; phpc: runtime error, `crypt()` only handles invalid `_` salt fallback. | `compiler/src/interpreter.rs::call_crypt`; `compiler/tests/crypt_builtin.rs` | Low |
| 2 | `ext/standard/tests/strings/bug78346.phpt` | 170 B | PHP: `strip_tags('<?= \'<?= 1 ?>\' ?>2')` => `"2"`; phpc leaves `"' ?>2"`. | `call_strip_tags`, `strip_tags_bytes`, `strip_tags_span` PHP/PI terminator handling | Low |
| 3 | `ext/standard/tests/strings/bug62443.phpt` | 180 B | PHP: malformed `$5$`/`$6$` salts with NUL return safely and script prints `OK!`; phpc aborts on unsupported `crypt()`. | `call_crypt` malformed SHA salt branch | Low/Med |
| 4 | `ext/standard/tests/strings/bug61374.phpt` | 189 B | PHP leaves `&OElig;` unchanged for `html_entity_decode(..., 'ISO-8859-1')`; phpc decodes it to `Œ`. | `call_html_entity_decode`, `html_decode_bytes`, `html_decode_entity_at`, encoding gate helpers | Med |
| 5 | `ext/standard/tests/strings/bug72146.phpt` | 189 B | PHP returns array with `"ABC123"`; phpc panics in Rust on `start + PHP_INT_MAX` overflow. | `call_substr_replace`, `substr_replace_scalar_string` saturating length math | Low |
| 6 | `ext/standard/tests/strings/gh18897.phpt` | 190 B | PHP prints `3` for both `%.f` and `%.0f`; phpc throws `ValueError: Unknown format specifier "f"` for `%.f`. | `parse_sprintf_placeholder`, precision parser (`.` with zero digits), `format_sprintf_value` | Low |
| 7 | `ext/standard/tests/strings/bug38322.phpt` | 210 B | PHP `sscanf("a ", "%1$s", $str)` assigns one value and returns `int(1)`; phpc rejects `$` as a bad scan conversion. | `call_sscanf_direct`, `parse_fscanf_format`, scanf positional/width parsing | Med |
| 8 | `ext/standard/tests/strings/bug69522.phpt` | 233 B | PHP warns `unpack(): Type h: integer overflow`; phpc reports an impossible huge not-enough-input count. | `parse_pack_format_items`, `call_unpack`, `unpack_has_bytes` repeat overflow diagnostics | Low/Med |
| 9 | `ext/standard/tests/strings/gh15552.phpt` | 237 B | PHP throws `ValueError: "%n$" argument index out of range`; phpc reports bad scan conversion `$`. | `parse_fscanf_format` positional-argument index validation | Med |
| 10 | `ext/standard/tests/strings/bug50052.phpt` | 257 B | PHP computes deterministic MD5-crypt output for `$1$f+uslYF01$`; phpc aborts on unsupported `crypt()`. | `call_crypt` MD5 `$1$` salt path, possibly shared crypt backend/constants | High |
| 11 | `ext/standard/tests/strings/bug29119.phpt` | 263 B | PHP decodes UTF-8 named entities to hex `e28082...e282ac`; phpc leaves entity text (`26656e7370...`). | `HTML_ENTITY_EXTRA_TRANSLATIONS`, `html_decode_entity_at`, UTF-8 entity coverage | Med |
| 12 | `ext/standard/tests/strings/bug68996.phpt` | 274 B | PHP emits display warning for invalid-UTF-8 filename `fopen("\xfc\x63", "r")`; phpc runtime-errors that path must be string, got string. | `call_fopen`, `filesystem_filename_argument`, binary-string path/display warning path | Med |
| 13 | `ext/standard/tests/strings/htmlentities12.phpt` | 281 B | PHP with `default_charset=ISO-8859-1` returns `&auml;&ouml;&uuml;`; phpc returns replacement characters. | `call_htmlentities`, `htmlentities_encode_bytes`, charset/default-charset helpers | Med |
| 14 | `ext/standard/tests/strings/gh10940.phpt` | 282 B | PHP warning says `only 12 were provided`; phpc warning says `only 12 was provided`. | `unpack_has_bytes` warning pluralization | Low |
| 15 | `ext/standard/tests/strings/htmlentities11.phpt` | 286 B | PHP with `default_charset=ISO-8859-15` maps bytes to `&OElig;&oelig;&Yuml;`; phpc returns replacement characters. | `htmlentities_encode_bytes`, ISO-8859-15 entity table/default charset | Med |
| 16 | `ext/standard/tests/strings/bug43957.phpt` | 301 B | PHP emits 8.2 deprecation and outputs `abc?`; phpc says `Call to undefined function utf8_decode()`. | function registry plus bounded `utf8_decode()`/deprecation helper | Med |
| 17 | `ext/standard/tests/strings/htmlentities07.phpt` | 303 B | PHP uses `mb_internal_encoding()` (`ISO-8859-1`) and encodes `äöü`; phpc lacks `mb_internal_encoding()`. | bounded `mb_internal_encoding()` metadata/runtime plus `htmlentities` default encoding | Med/High |
| 18 | `ext/standard/tests/strings/crypt_blowfish_variation2.phpt` | 304 B | PHP returns `*0` for unsupported/malformed Blowfish salt shape and prints `OK`; phpc aborts on unsupported `crypt()`. | `call_crypt` Blowfish salt validation/fallback | Med |
| 19 | `ext/standard/tests/strings/htmlentities06.phpt` | 308 B | PHP uses `mb_internal_encoding()` (`ISO-8859-15`) and encodes bytes to `&OElig;&oelig;&Yuml;`; phpc lacks `mb_internal_encoding()`. | bounded `mb_internal_encoding()` plus ISO-8859-15 html entity table | Med/High |
| 20 | `ext/standard/tests/strings/htmlentities01.phpt` | 317 B | PHP cp1252 encodes `&sbquo;&dagger;&trade;&Yuml;` and `&euro;&cent;...`; phpc emits raw/replacement bytes for unsupported cp1252 entries. | `htmlentities_encode_bytes`, cp1252 entity mapping | Med |
| 21 | `ext/standard/tests/strings/bug47322.phpt` | 326 B | First two `sscanf()` rows match; third should leave prior `$b/$c` values (`[15.1111][1][58.2]`), phpc overwrites them to empty/null. | `FscanfFormat::scan_line`, `call_sscanf_direct` by-ref assignment of failed conversions | Med |
| 22 | `ext/standard/tests/strings/bug71190.phpt` | 327 B | PHP accepts string offset `"1"` and leaves replacement array unchanged; phpc rejects offset as non-int before completing. | `call_substr_replace`, `substr_replace_int_argument`, scalar int coercion | Low/Med |
| 23 | `ext/standard/tests/strings/bug72433.phpt` | 341 B | Both return `bool(false)`, but PHP warns error at offset `13`; phpc warns offset `9`. | `call_unserialize_builtin` / unserialize parser offset accounting | Med/High |
| 24 | `ext/standard/tests/strings/bug38770.phpt` | 350 B | PHP supports `pack/unpack('N')` and `l`; phpc rejects pack format code `N`. | `parse_pack_format_items`, `pack_integer_bytes`, `pack_fixed_item_size`, `unpack_fixed_value` | Low/Med |
| 25 | `ext/standard/tests/strings/crypt_chars.phpt` | 359 B | PHP returns four DES/extended-DES crypt strings; phpc aborts on unsupported `crypt()`. | `call_crypt` DES / extended-DES backend | High |

## Notes and near misses

- `ext/standard/tests/strings/bug61764.phpt` (378 B) is just outside the top 25 and likely lower risk than the broader DES crypt row: add `pack('L')`/`unpack('I')` unsigned-long coverage.
- Many adjacent `htmlentities*` rows share the same charset/default-charset gaps; a single bounded encoding-table lane could knock out several, but risk rises quickly outside cp1252/ISO-8859-*.
- `ext/standard/tests/url/get_headers_error_003.phpt` is the smallest URL failure observed in this slice (619 B), but the immediate blocker is parser/runtime array spread before the test reaches `get_headers()`, so it is not as narrow as the string rows above.
- No implementation files were edited by this discovery worker.
