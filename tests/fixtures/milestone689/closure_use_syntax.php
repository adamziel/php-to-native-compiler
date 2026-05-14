<?php
function _wp_can_use_pcre_u_probe() {
    $utf8_pcre = null;
    $handler = function ($errno, $errstr) use (&$utf8_pcre) {
        $utf8_pcre = false;
        return false;
    };
    return "registered";
}

echo "closure syntax parsed";
