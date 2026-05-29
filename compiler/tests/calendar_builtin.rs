use php_compiler::run_source;

#[test]
fn calendar_constants_and_gregorian_julian_conversions_are_available() {
    let execution = run_source(
        r#"<?php
var_dump(defined("CAL_GREGORIAN"));
var_dump(function_exists("gregoriantojd"));
echo CAL_GREGORIAN, "|", CAL_JULIAN, "|", CAL_NUM_CALS, "|", CAL_DOW_LONG, "\n";
echo cal_days_in_month(CAL_GREGORIAN, 2, 2004), "|", cal_days_in_month(CAL_GREGORIAN, 2, 2003), "\n";
echo cal_days_in_month(CAL_JULIAN, 2, 1900), "|", cal_days_in_month(CAL_JULIAN, 2, 1901), "\n";
echo gregoriantojd(1, 1, 1970), "|", jdtogregorian(2440588), "\n";
echo gregoriantojd(1, 1, 1582), "|", jdtogregorian(2298874), "\n";
echo juliantojd(1, 1, 1970), "|", jdtojulian(2440588), "\n";
echo jdtogregorian(1), "|", jdtojulian(1), "\n";
echo jddayofweek(2440588, CAL_DOW_DAYNO), "|", jddayofweek(2440588, CAL_DOW_LONG), "|", jddayofweek(2440588, CAL_DOW_SHORT), "\n";
echo unixtojd(0), "|", jdtounix(2440588), "\n";
"#,
    )
    .unwrap();

    assert_eq!(
        execution.stdout,
        "bool(true)\nbool(true)\n0|1|4|1\n29|28\n29|28\n2440588|1/1/1970\n2298874|1/1/1582\n2440601|12/19/1969\n11/25/-4714|1/2/-4713\n4|Thursday|Thu\n2440588|0\n"
    );
    assert_eq!(execution.stderr, "");
    assert_eq!(execution.exit_code, 0);
}
