<?php
function _wp_scan_utf8(string $bytes, int &$at, int &$invalid_length, ?int $max_bytes = null, ?int $max_code_points = null, ?bool &$has_noncharacters = null): int {
    return 0;
}

function union_result($value): int|string {
    return $value;
}

function intersection_param(Iterator&Countable $value) {
    return $value;
}

echo "scan=", function_exists("_wp_scan_utf8"), "\n";
echo "union=", function_exists("union_result"), "\n";
echo "intersection=", function_exists("intersection_param");
