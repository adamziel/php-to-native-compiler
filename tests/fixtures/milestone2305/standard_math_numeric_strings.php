<?php
var_dump(is_finite("1e9999"));
var_dump(sqrt("1e9999"));
echo ceil(" 039 "), "|", floor("-2.7"), "|", abs("-4.5"), "\n";
$invalids = ["123abc", "INF", "NAN"];
foreach ($invalids as $index => $value) {
    try {
        is_finite($value);
    } catch (TypeError $e) {
        echo $value, ":", $e->getMessage();
        if ($index + 1 < count($invalids)) {
            echo "\n";
        }
    }
}
