# Timelib Date/Time Backend Boundary

PTN's `ext/date` surface must converge on PHP timelib semantics. The backend
boundary is one shared date/time engine for scalar functions, `DateTime`,
`DateTimeImmutable`, `DateTimeZone`, `DateInterval`, formatting, timezone data,
relative strings, and diagnostics. Do not add independent local parsers for
new date rows.

Current implementation status:

- `src/backend/runtime/internals_internal_functions.c` has bounded local
  helpers for Gregorian arithmetic, date formatting, `strtotime()` subsets,
  `date_parse()` subsets, `DateInterval` ISO specs, and DateTime object state.
- Timezone support is a small hand-maintained identifier table plus approximate
  DST and abbreviation rules. It is useful for the existing native smoke rows,
  but it is not a PHP timezone database.
- `date_parse_from_format()` and `DateTime::createFromFormat()` rows remain
  classifier-blocked because timelib owns PHP's format-token parser,
  normalization, warnings, errors, and fractional-second behavior.
- `DateTimeZone::getTransitions()` and full abbreviation/identifier listings
  require timelib timezone data, not row-shaped local arrays.
- `DateInterval::createFromDateString()` and relative interval strings require
  the same timelib relative parser as `strtotime()` and `DateTime::modify()`.

## Target Shape

Add a single C runtime adapter around a vendored or linked timelib-compatible
library. The adapter should expose PTN-owned functions whose names make the
semantic boundary explicit:

```c
PtnDateParseResult ptn_timelib_parse_datetime(
    PtnRuntime *runtime,
    PtnStringOperand input,
    const char *default_timezone,
    size_t line);

PtnDateParseResult ptn_timelib_parse_from_format(
    PtnRuntime *runtime,
    PtnStringOperand format,
    PtnStringOperand input,
    const char *default_timezone,
    size_t line);

PtnDateIntervalData ptn_timelib_parse_interval(
    PtnRuntime *runtime,
    PtnStringOperand input,
    size_t line);

PtnTimezoneData ptn_timelib_timezone_lookup(
    PtnRuntime *runtime,
    PtnStringOperand name,
    size_t line);
```

The returned structs must be PTN-owned boxed-runtime data, not timelib pointers
escaping into user objects. Timelib objects should be copied into
`PtnDateTimeData`, `PtnDateTimeZoneData`, and `PtnDateIntervalData`, and freed
at the adapter boundary.

## Required Routing

These runtime paths should all call through the adapter before more PHPT rows
are promoted:

- parsing and normalization:
  `ptn_datetime_parse_date_string()`,
  `ptn_datetime_parse_textual_date_string()`,
  `ptn_datetime_parse_partial_textual_date_string()`,
  `ptn_datetime_parse_relative_seconds()`, and
  `ptn_internal_date_parse()`;
- format parsing:
  `date_parse_from_format()`, `DateTime::createFromFormat()`, and
  `DateTimeImmutable::createFromFormat()`;
- object mutation:
  `DateTime::__construct()`, `DateTimeImmutable::__construct()`,
  `DateTimeInterface::modify()`, `date_modify()`, `date_add()`,
  `date_sub()`, `setDate()`, `setISODate()`, `setTime()`, and
  `setTimestamp()`;
- timezone data:
  `DateTimeZone::__construct()`, `timezone_open()`,
  `timezone_identifiers_list()`, `DateTimeZone::listIdentifiers()`,
  `timezone_abbreviations_list()`, `DateTimeZone::listAbbreviations()`,
  `timezone_name_from_abbr()`, `timezone_offset_get()`,
  `DateTimeZone::getOffset()`, `timezone_transitions_get()`, and
  `DateTimeZone::getTransitions()`;
- interval data:
  `DateInterval::__construct()`,
  `DateInterval::createFromDateString()`,
  `date_interval_create_from_date_string()`, DateTime `diff()`, DateTime
  `add()`/`sub()`, and interval serialization properties;
- diagnostics:
  parse warnings/errors, `date_get_last_errors()`, bad timezone names,
  malformed interval strings, range checks, and object initialization errors.

## Diagnostics Contract

The adapter must preserve PHP's warning and error arrays:

- `date_parse()` and `date_parse_from_format()` populate
  `warning_count`, `warnings`, `error_count`, and `errors`.
- `date_get_last_errors()` returns the last timelib warning/error state or
  `false` when there were no warnings/errors.
- DateTime constructors and factory methods bridge parse failures to PHP's
  current exception or warning behavior for the called API.
- Diagnostic offsets must come from the parser that consumed the original byte
  string. Do not recompute offsets after UTF-8 or C-string normalization.

## Bounded Helpers

Until the adapter lands, the following helpers are explicitly bounded and must
not grow into a second date engine:

- hand-written textual-date and relative-date parsers;
- hand-written timezone identifier, abbreviation, offset, and DST tables;
- local DateInterval relative-string parsing;
- local parse warning/error synthesis outside existing smoke-test support.

Any new compatibility work in this area should either extend the timelib
adapter or keep its temporary status documented in this file.

## Focused Evidence

The focused PHPT evidence for this boundary is:

```text
tools/phpt-ptn-hvvb4.2-timelib-date-boundary-row-pack.txt
```

That manifest intentionally mixes currently supported scalar/object rows with
blocked parser, timezone-transition, relative-string, and interval rows. It is
the acceptance frontier for replacing the local helpers with timelib-compatible
semantics.
