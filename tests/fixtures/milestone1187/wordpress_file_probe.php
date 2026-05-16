<?php
define("ABSPATH", __DIR__ . "/");

$target = ABSPATH . "wp-config.php";

echo is_file($target) ? "config-file" : "missing-config";

