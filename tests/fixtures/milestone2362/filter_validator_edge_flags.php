<?php
foreach (["0x8000000000000000", "0xffffffffffffffff", "0x10000000000000000"] as $value) {
    var_dump(filter_var($value, FILTER_VALIDATE_INT, FILTER_FLAG_ALLOW_HEX));
}

var_dump(defined("FILTER_FLAG_GLOBAL_RANGE"));
var_dump(FILTER_FLAG_GLOBAL_RANGE);
var_dump(FILTER_FLAG_HOSTNAME);

foreach (["0.0.0.0", "185.85.0.29", "::", "64:ff9b::"] as $ip) {
    var_dump(filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_GLOBAL_RANGE));
}

foreach (["169.254.0.0", "224.0.0.0"] as $ip) {
    var_dump(filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_IPV4 | FILTER_FLAG_NO_RES_RANGE));
}

foreach (["http://t[est@127.0.0.1", "http://test@[::1]"] as $url) {
    var_dump(filter_var($url, FILTER_VALIDATE_URL));
}

if (filter_var("a-.bc.com", FILTER_VALIDATE_DOMAIN, FILTER_FLAG_HOSTNAME) === false) {
    echo "bad-domain-ok";
} else {
    echo "bad-domain-fail";
}
