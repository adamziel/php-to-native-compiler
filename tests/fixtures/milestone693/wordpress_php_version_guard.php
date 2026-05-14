<?php
function _wp_php_version_probe() {
    if (PHP_VERSION_ID < 70224) {
        return "unsupported";
    }

    return "supported";
}

echo _wp_php_version_probe();
