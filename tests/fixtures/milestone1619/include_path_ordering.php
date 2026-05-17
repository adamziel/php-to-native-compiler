<?php
$old = set_include_path(__DIR__ . "/include_path_lib");
$first = include "same_name.inc";
echo "|first=" . $first;

set_include_path(__DIR__ . "/missing_path");
$second = include "source_fallback.inc";
echo "|second=" . $second;
