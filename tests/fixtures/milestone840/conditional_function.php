<?php
if (!function_exists('wp_redirect')) {
    function wp_redirect($location, $status = 302) {
        echo 'redirect:' . $location . '|' . $status;
        return true;
    }
}

echo function_exists('wp_redirect') ? 'declared|' : 'missing|';
wp_redirect('/wp-admin/install.php');
