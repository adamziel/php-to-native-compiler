# Known-Failure No-Regression Gate Current-Base Repair

Developer: developer-290
Repair worklane/card: 2010
Original card: 1946, referencing worklane 1883
Owner blocker card: 1808

## Repair Summary

`origin/master` already contains the known-failure gate script from the earlier
current-base resolver, so this repair keeps that implementation and applies only
the remaining non-conflicting card 1946 deltas:

- align the focused unit-test `metric_samples` schema with the live harness
  table;
- prove `--no-write-metadata` leaves the database untouched;
- keep committed status/progress metric displays on
  `accepted_public_phpt_passes / pinned_public_runnable_denominator`.

The old generated status row churn from `work/developer-284` was intentionally
not carried forward.

## Active Baseline

The tracked gate script pins the Manhole-approved known-red baseline to exactly:

- `tests::native_closure_invoke_helpers_bridge_call_arguments_to_call_results`
- `tests::native_magic_method_lookup_rejects_malformed_signature_metadata_before_fallback`

Removal/update condition: remove or update the baseline only after card 1808 is
reviewed/integrated or after a new scheduler-approved quarantine baseline is
recorded.

## Source Caveat

The checkout still does not contain the live harness zipapp or durable scheduler
source that decides the running status generator. The tracked script remains the
deterministic repository artifact for integrators to wire into the live
control-plane path that records `test_runs`.
