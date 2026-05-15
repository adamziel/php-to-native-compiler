<?php
define("ABSPATH", __DIR__ . "/");
define("WP_CONTENT_DIR", __DIR__);

if (!function_exists("mysqli_connect") && !file_exists(WP_CONTENT_DIR . "/db.php")) {
    echo "needs-db-extension";
} else {
    echo "ok";
}

