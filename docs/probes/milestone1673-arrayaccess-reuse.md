# Milestone 1673 ArrayAccess Reuse Probes

These probes classify the alias/reuse nuance left after Milestone 1672 copied
`ArrayAccess` bucket reference-slot propagation. They intentionally do not
claim support for the failing shapes.

## PHP-Compatible Controls

`tests/fixtures/milestone1673c/arrayaccess_bucket_reuse_controls.php` matches
system PHP and remains eligible for `--compare-php`.

Observed PHP and phpc output:

```text
outer-bucket-reuse:seed|seed|bucket:reused
callback-name-reuse:new-local|seed|after-callback-reuse
two-distinct-refs:first:mutated|second:mutated|first:mutated|second:mutated
```

This classifies three non-failing shapes:

- Replacing the outer copied `$bucket` variable detaches subsequent writes
  from the original callback slot.
- `unset($callback); $callback = ...` detaches the direct callback variable
  name while the copied bucket reference slot can still write to the backing
  hook bucket.
- Two distinct nested reference slots in one copied bucket propagate
  independently.
- `tests/fixtures/milestone1673c/arrayaccess_bucket_foreach_callback_reuse_control.php`
  shows that reusing the lingering by-reference foreach `$callback` variable
  by assigning a new array still preserves the first nested reference write and
  does not overwrite the original callback slot.

## Former phpc Mismatch Fixed

`tests/fixtures/milestone1673c/arrayaccess_bucket_parameter_reuse_alias_gap.php`
is now PHP-comparable. System PHP leaves the original callback slot at
`param:first|param:first|param:first` after the by-value helper parameter is
replaced, and phpc now matches that output:

```text
param:first|param:first|param:first
```

That means the mirrored copied-bucket path writes through before replacement,
then detaches when the helper parameter variable is replaced, so the
replacement array's later nested write stays local to the helper.

No mismatch was observed for the lingering foreach callback-variable reuse
shape under the current source.
