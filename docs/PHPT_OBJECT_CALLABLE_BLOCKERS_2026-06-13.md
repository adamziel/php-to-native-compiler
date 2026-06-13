# PHPT Object/Class/Callable Blocker Map 2026-06-13

Scope: `ptn-qsmv.5` broad object/class/callable metadata slice.

Evidence came from focused PHPT probes against `/home/claude/php-src-phpt` at
corpus revision `8c63ec400ce8e07c57a8d9499317b96a8beafb8b`:

- `.runtime/phpt-progress/run-20260613T140825Z.log`: 114 selected, 112 runnable,
  17 passed, 95 failed, 2 classified harness exclusions.
- `.runtime/phpt-progress/run-20260613T141331Z.log`: 79 selected, 79 runnable,
  8 passed, 71 failed.
- `.runtime/phpt-progress/run-20260613T141857Z.log`: 5 selected, 5 runnable,
  2 passed, 3 failed.

The bounded manifest gained 25 verified passing rows in this slice. The 50 rows
below remain runnable PHPT semantic failures and are grouped by generic blocker
with follow-up bead IDs.

## ptn-12on: call_user_func/call_user_func_array edges

- `Zend/tests/call_user_functions/bug32290.phpt`
- `Zend/tests/call_user_functions/bug66719.phpt`
- `Zend/tests/call_user_functions/call_user_func_001.phpt`
- `Zend/tests/call_user_functions/call_user_func_002.phpt`
- `Zend/tests/call_user_functions/call_user_func_003.phpt`
- `Zend/tests/call_user_functions/call_user_func_005.phpt`
- `Zend/tests/call_user_functions/call_user_func_006.phpt`
- `Zend/tests/call_user_functions/call_user_func_007.phpt`
- `Zend/tests/call_user_functions/call_user_func_008.phpt`
- `Zend/tests/call_user_functions/call_user_func_009.phpt`
- `Zend/tests/call_user_functions/call_user_func_array_invalid_type.phpt`
- `Zend/tests/call_user_functions/call_user_func_by_ref.phpt`

## ptn-h47t: `::class` and class-name scalar semantics

- `Zend/tests/class_name/bug66811.phpt`
- `Zend/tests/class_name/bug69754.phpt`
- `Zend/tests/class_name/bug77530.phpt`
- `Zend/tests/class_name/class_name_as_scalar.phpt`
- `Zend/tests/class_name/class_name_as_scalar_error_001.phpt`
- `Zend/tests/class_name/class_name_as_scalar_error_002.phpt`
- `Zend/tests/class_name/class_name_as_scalar_error_003.phpt`
- `Zend/tests/class_name/class_name_as_scalar_error_004.phpt`
- `Zend/tests/class_name/class_name_as_scalar_error_005.phpt`
- `Zend/tests/class_name/class_on_object.phpt`

## ptn-98lj: first-class callable syntax and metadata

- `Zend/tests/first_class_callable/constexpr/basic.phpt`
- `Zend/tests/first_class_callable/constexpr/case_insensitive.phpt`
- `Zend/tests/first_class_callable/constexpr/default_args.phpt`
- `Zend/tests/first_class_callable/constexpr/static_call.phpt`
- `Zend/tests/first_class_callable/constexpr/static_call_self.phpt`
- `Zend/tests/first_class_callable/constexpr/userland.phpt`
- `Zend/tests/first_class_callable/first_class_callable_001.phpt`
- `Zend/tests/first_class_callable/first_class_callable_002.phpt`
- `Zend/tests/first_class_callable/first_class_callable_003.phpt`
- `Zend/tests/first_class_callable/first_class_callable_004.phpt`
- `Zend/tests/first_class_callable/first_class_callable_005.phpt`
- `Zend/tests/first_class_callable/first_class_callable_006.phpt`

## ptn-0pys: interfaces, traits, and class modifiers

- `Zend/tests/class_exists_003.phpt`
- `Zend/tests/constants/class_constants_004.phpt`
- `Zend/tests/objects/objects_012.phpt`
- `Zend/tests/objects/objects_013.phpt`
- `Zend/tests/objects/objects_014.phpt`
- `Zend/tests/objects/objects_018.phpt`

## ptn-nya1: class constants, static properties, and static variables

- `Zend/tests/class_properties_const.phpt`
- `Zend/tests/class_properties_dynamic.phpt`
- `Zend/tests/constants/class_constants_001.phpt`
- `Zend/tests/constants/class_constants_002.phpt`
- `Zend/tests/static_variables/static_variable.phpt`
- `Zend/tests/varSyntax/staticMember.phpt`

## ptn-8fip: object magic, visibility, cloning, and comparison

- `Zend/tests/object_array_cast.phpt`
- `Zend/tests/object_handlers.phpt`
- `Zend/tests/objects/objects_001.phpt`
- `Zend/tests/objects/objects_021.phpt`
