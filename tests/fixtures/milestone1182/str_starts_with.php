<?php
$_SERVER['SCRIPT_FILENAME'] = '/var/www/html/wp-admin/admin-ajax.php';
$call = 'str_starts_with';
echo function_exists($call) ? 'yes' : 'no';
echo '|';
echo is_callable($call) ? 'callable' : 'missing';
echo '|';
echo str_starts_with($_SERVER['SCRIPT_FILENAME'], '/var/www/html/wp-admin') ? 'admin' : 'other';
echo '|';
echo $call('wp-content/plugins/example.php', 'wp-content') ? 'prefix' : 'missing';
echo '|';
echo str_starts_with('index.php', '') ? 'empty' : 'missing';
