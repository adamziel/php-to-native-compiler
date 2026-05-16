<?php
echo is_array($_COOKIE) ? "array" : "missing";
echo "|";
echo isset($_COOKIE["wordpress_test_cookie"]) ? "cookie" : "empty";

function wp_cookie_probe() {
    $_COOKIE["wordpress_test_cookie"] = "WP Cookie check";
}

wp_cookie_probe();
echo "|";
echo $_COOKIE["wordpress_test_cookie"];
