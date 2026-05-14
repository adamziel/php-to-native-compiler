<?php
$mode = "ARRAY_FILTER_USE_KEY";
$missing = "APP_NAME";

echo defined("CASE_LOWER") ? "1" : "0";
echo defined("CASE_UPPER") ? "1" : "0";
echo defined("ARRAY_FILTER_USE_BOTH") ? "1" : "0";
echo defined("ARRAY_FILTER_USE_KEY") ? "1" : "0";
echo defined("SORT_STRING") ? "1" : "0";
echo defined("MISSING_CONST") ? "1" : "0";
echo defined($mode) ? "1" : "0";
echo defined($missing) ? "1" : "0";
