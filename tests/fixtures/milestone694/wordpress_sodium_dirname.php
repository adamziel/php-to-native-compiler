<?php
function _wp_sodium_autoload_probe($file) {
    return dirname($file) . "/autoload-php7.php";
}

echo _wp_sodium_autoload_probe("/wordpress/wp-includes/sodium_compat/autoload.php");
