<?php
$old = set_include_path(__DIR__ . "/include_path_lib");
$result = include "wp_loader.inc";
echo "result=" . $result;
echo "|old=" . $old;
echo "|loaded=" . $loaded;

