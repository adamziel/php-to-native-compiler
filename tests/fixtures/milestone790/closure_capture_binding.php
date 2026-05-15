<?php
$utf8_pcre = true;
set_error_handler(
    function ($errno, $errstr) use (&$utf8_pcre) {
        $utf8_pcre = false;
        return false;
    },
    E_WARNING
);
echo "registered";
