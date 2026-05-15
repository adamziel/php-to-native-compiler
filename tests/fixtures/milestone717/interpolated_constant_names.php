<?php
$constant = "RUNTIME";
define("APP_RUNTIME", "ok");
echo defined("APP_$constant") ? "1" : "0";
echo "|", constant("APP_$constant"), "\n";
$constant = "MISSING";
echo defined("APP_$constant") ? "1" : "0", "\n";
echo "literal:\$constant", "\n";
