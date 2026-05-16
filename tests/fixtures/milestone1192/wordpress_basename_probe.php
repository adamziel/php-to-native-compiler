<?php
function wp_basename_probe($path) {
    return basename($path, ".php");
}

echo wp_basename_probe("/wordpress/wp-includes/plugin.php");
