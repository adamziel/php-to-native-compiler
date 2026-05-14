<?php
const ABSPATH = '';
const WPINC = 'wp-includes';

require ABSPATH . WPINC . '/load.php';

echo wp_loaded_label(), "\n";
echo WP_Loaded::name();
