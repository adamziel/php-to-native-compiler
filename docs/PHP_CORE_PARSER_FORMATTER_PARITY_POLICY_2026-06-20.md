# PHP-Core Parser/Formatter Parity Policy

Issue: `ptn-hvvb4.7`
Date: 2026-06-20

This record caps complex PHP-core parser and formatter surfaces that are
currently implemented locally in PTN. The goal is to keep these surfaces from
growing as one-off hand-written compatibility patches without an explicit owner,
source-of-truth policy, and differential PHPT gate.

## Policy

Use one of three ownership modes for each surface:

| Mode | Use When | PTN Rule |
| --- | --- | --- |
| External delegation | PHP itself delegates the semantic source of truth to an external library or host API. | Wrap the same boundary and keep PHP diagnostics/state in PTN. Do not swap in unrelated libraries. |
| Generated upstream data | PHP owns stable constants, flags, names, defaults, or lookup tables. | Generate PTN tables from php-src instead of maintaining local copies by hand. |
| PTN-core semantics | Behavior depends on PHP zvals, references, object/class metadata, stream state, INI state, or VM diagnostics. | Keep implementation in PTN runtime code and require a focused PHPT parity manifest before expanding support. |

## Decisions

| Surface | Current Evidence | Decision | Cap / Gate | Follow-Up |
| --- | --- | --- | --- | --- |
| `serialize()` / `unserialize()` | `ptn_serialize_*` starts near `src/backend/runtime/internals_internal_functions.c:5703`; `ptn_internal_unserialize()` starts at `src/backend/runtime/internals_internal_functions.c:8436`; existing row packs include `tools/phpt-ptn-w17z25-serialize-reference-identity-row-pack.txt` and `tools/phpt-ptn-w17z26-serializable-spl-unserialize-row-pack.txt`. | PTN-core semantics. The format is inseparable from PHP reference IDs, object identity, magic hooks, incomplete classes, Serializable/SPL hydration, diagnostics, and runtime allocation guards. No third-party parser is an acceptable source of truth. | New work must run or extend `tools/phpt-ptn-hvvb4.7-serialize-reference-parity-manifest.txt` and prove reference/object identity behavior, not just byte-string syntax. | No external migration. Refactors may split code into a serializer module, but behavior remains PTN runtime-owned. |
| JSON encode/decode and error state | JSON flags and UTF handling appear around `src/backend/runtime/internals_internal_functions.c:8777`; `ptn_internal_json_encode()` starts at `src/backend/runtime/internals_internal_functions.c:9325`; `ptn_internal_json_decode()` starts at `src/backend/runtime/internals_internal_functions.c:9993`; JSON constants are registered near `src/backend/runtime/internals_internal_functions.c:57784`. | Mixed: PTN-core semantics for value traversal, UTF diagnostics, recursion, depth, JsonSerializable, error state, and exceptions; generated upstream data for constants, allowed flag masks, error names, and error-message text. PHP owns ext/json and does not delegate this surface to a general JSON library. | New work must run or extend `tools/phpt-ptn-hvvb4.7-json-flags-utf-manifest.txt`. | `ptn-9vxez`: generate JSON constants and error metadata from php-src. |
| CSV parsing/writing (`fgetcsv`, `str_getcsv`, `fputcsv`, SPL CSV control) | Shared `ptn_csv_*` helpers start near `src/backend/runtime/internals_internal_functions.c:38367`; `ptn_internal_fgetcsv()` starts at `src/backend/runtime/internals_internal_functions.c:38665`; `ptn_internal_str_getcsv()` starts at `src/backend/runtime/internals_internal_functions.c:38733`; `ptn_internal_fputcsv()` starts at `src/backend/runtime/internals_internal_functions.c:38822`; existing evidence includes `tools/phpt-ptn-jrzu-csv-stream-parity-row-pack.txt`. | PTN-core semantics. PHP standard owns this parser/writer behavior, and it is tied to stream resources, `SplFileObject`, delimiter/enclosure/escape diagnostics, line-ending behavior, and deprecation timing. Generic CSV libraries are not parity owners. | New work must run or extend `tools/phpt-ptn-hvvb4.7-csv-edge-parity-manifest.txt`. | `ptn-93wbb`: extract CSV parser and writer into a dedicated parity runtime module. |
| Query encoding and query parsing (`http_build_query`, `parse_str`) | `ptn_internal_http_build_query()` starts at `src/backend/runtime/internals_internal_functions.c:18046`; `ptn_internal_parse_str()` starts at `src/backend/runtime/internals_internal_functions.c:23073`. | PTN-core semantics. The source of truth is PHP standard-extension behavior: array/object traversal, references, name mangling, `arg_separator.*`, null bytes, `max_input_vars`, and RFC1738/RFC3986 escaping. A URL library can only provide primitive percent-encoding if it exactly matches PHP's selected mode. | New work must run or extend `tools/phpt-ptn-hvvb4.7-query-encoding-manifest.txt`. | `ptn-hmgls`: extract query string encoding and `parse_str` into a parity runtime module. |
| Numeric and scalar formatting (`sprintf` family, float stringification, serialize precision) | `ptn_internal_sprintf_named()` starts at `src/backend/runtime/internals_internal_functions.c:25483`; serializer float formatting calls `ptn_format_runtime_serialize_float()` near `src/backend/runtime/internals_internal_functions.c:6242`; runtime float formatting helpers live in `src/backend/runtime/strings.c`. Existing broad evidence includes `tools/phpt-ptn-w17z21-formatting-row-pack.txt`. | PTN-core semantics with generated upstream data for INI defaults, constants, and stable formatter metadata where php-src owns data. Do not claim an external formatter library as parity owner unless php-src itself delegates the exact semantic boundary. | Keep formatter expansion behind focused PHPT rows from the formatting row pack. | `ptn-tzzlu`: generate locale and scalar-format metadata from php-src. |
| Locale (`setlocale`, `localeconv`, `LC_*`) | `ptn_internal_setlocale()` starts at `src/backend/runtime/internals_internal_functions.c:58930`; `ptn_internal_localeconv()` starts at `src/backend/runtime/internals_internal_functions.c:58989`; locale rows are in `tools/phpt-ptn-w17z21-formatting-row-pack.txt`. | External delegation for host locale state, because PHP delegates `setlocale()`/`localeconv()` to the platform C locale API. Generated upstream data is still required for PHP-visible constants and metadata. | Do not hard-code host-specific locale outputs beyond C/POSIX-stable expectations. Locale-sensitive rows need SKIPIF/precondition classification. | Covered by `ptn-tzzlu`. |

## Parity Manifests

Committed focused gates:

```text
tools/phpt-ptn-hvvb4.7-serialize-reference-parity-manifest.txt
tools/phpt-ptn-hvvb4.7-csv-edge-parity-manifest.txt
tools/phpt-ptn-hvvb4.7-json-flags-utf-manifest.txt
tools/phpt-ptn-hvvb4.7-query-encoding-manifest.txt
```

Suggested replay commands:

```sh
tools/run-bounded-phpt.sh tools/phpt-ptn-hvvb4.7-serialize-reference-parity-manifest.txt
tools/run-bounded-phpt.sh tools/phpt-ptn-hvvb4.7-csv-edge-parity-manifest.txt
tools/run-bounded-phpt.sh tools/phpt-ptn-hvvb4.7-json-flags-utf-manifest.txt
tools/run-bounded-phpt.sh tools/phpt-ptn-hvvb4.7-query-encoding-manifest.txt
```

These manifests are gates for future work, not a support claim that every row is
currently green.
