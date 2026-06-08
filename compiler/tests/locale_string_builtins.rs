use php_compiler::run_source;

#[test]
fn nl_langinfo_reports_c_locale_day_month_and_radix_entries() {
    let execution = run_source(
        r#"<?php
setlocale(LC_ALL, "C");
var_dump(nl_langinfo(ABDAY_2));
var_dump(nl_langinfo(DAY_4));
var_dump(nl_langinfo(ABMON_7));
var_dump(nl_langinfo(MON_4));
var_dump(nl_langinfo(RADIXCHAR));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "string(3) \"Mon\"\n\
string(9) \"Wednesday\"\n\
string(3) \"Jul\"\n\
string(5) \"April\"\n\
string(1) \".\"\n"
    );
    assert_eq!(execution.exit_code, 0);
}

#[test]
fn nl_langinfo_unknown_items_warn_and_return_false() {
    let execution = run_source(
        r#"<?php
var_dump(nl_langinfo(999999));
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "Warning: nl_langinfo(): Item '999999' is not valid in Command line code on line 2\nbool(false)\n"
    );
    assert_eq!(execution.exit_code, 0);
}
