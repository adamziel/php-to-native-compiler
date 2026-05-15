<?php
$required_php_version = "7.2.24";
$php_version = PHP_VERSION;
if (version_compare($required_php_version, $php_version, ">")) {
    echo "unsupported";
} else {
    echo "supported";
}
