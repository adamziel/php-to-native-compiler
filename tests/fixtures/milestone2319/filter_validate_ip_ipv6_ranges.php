<?php
foreach ([
    ["FC00::1", FILTER_FLAG_IPV6 | FILTER_FLAG_NO_PRIV_RANGE],
    ["fdff:ffff:ffff:ffff:ffff:ffff:ffff:ffff", FILTER_FLAG_IPV6 | FILTER_FLAG_NO_PRIV_RANGE],
    ["::", FILTER_FLAG_IPV6 | FILTER_FLAG_NO_RES_RANGE],
    ["::1", FILTER_FLAG_NO_RES_RANGE],
    ["0:0:0:0:0:0:0:1", FILTER_FLAG_NO_RES_RANGE],
    ["fe80:5:6::1", FILTER_FLAG_IPV6 | FILTER_FLAG_NO_RES_RANGE],
    ["::ffff:0:1", FILTER_FLAG_NO_RES_RANGE],
    ["2001:0db8::1", FILTER_FLAG_IPV6 | FILTER_FLAG_NO_RES_RANGE],
    ["2001:0010::1", FILTER_FLAG_IPV6 | FILTER_FLAG_NO_RES_RANGE],
    ["240b:0010::1", FILTER_FLAG_IPV6 | FILTER_FLAG_NO_RES_RANGE],
    ["::ffff:192.168.1.1", FILTER_FLAG_NO_PRIV_RANGE],
] as $case) {
    var_dump(filter_var($case[0], FILTER_VALIDATE_IP, $case[1]));
}
echo "done";
