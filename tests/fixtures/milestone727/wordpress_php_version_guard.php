<?php
function _wp_php_version_string_probe() {
    if (!defined("PHP_VERSION")) {
        return "missing";
    }

    if (PHP_VERSION === "") {
        return "empty";
    }

    return "available";
}

echo _wp_php_version_string_probe();
