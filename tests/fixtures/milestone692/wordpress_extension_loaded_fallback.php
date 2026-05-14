<?php
function _wp_utf8_encode_probe($string) {
    if (extension_loaded("mbstring")) {
        return "mbstring";
    } else {
        return "fallback:" . $string;
    }
}

echo _wp_utf8_encode_probe("Ada");
