<?php
function _wp_alternate_if_probe($utf8_pcre) {
    if ($utf8_pcre):
        if (function_exists("strlen")):
            return "pcre";
        else:
            return "fallback";
        endif;
    endif;

    return "miss";
}

echo _wp_alternate_if_probe(true);
