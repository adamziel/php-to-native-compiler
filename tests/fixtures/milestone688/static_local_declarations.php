<?php
function _wp_can_use_pcre_u_probe($set = null) {
    static $utf8_pcre = null;

    if (isset($set)) {
        return $utf8_pcre;
    }

    if (isset($utf8_pcre)) {
        return $utf8_pcre;
    }

    $utf8_pcre = true;
    return $utf8_pcre;
}

echo _wp_can_use_pcre_u_probe() ? "yes" : "no";
echo "\n";
echo _wp_can_use_pcre_u_probe(false) ? "cached" : "miss";
