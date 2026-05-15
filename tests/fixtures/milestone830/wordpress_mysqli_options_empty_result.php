<?php
$handle = mysqli_init();
mysqli_real_connect($handle, 'localhost', 'user', 'pass', null, 3306, null, 0);
$autoload = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options WHERE autoload IN ( 'yes', 'on', 'auto-on', 'auto' )");
$fallback = mysqli_query($handle, 'SELECT option_name, option_value FROM wp_options');
echo $autoload === false ? 'autoload-empty' : 'autoload-result';
echo '|';
echo $fallback === false ? 'fallback-empty' : 'fallback-result';
echo '|';
echo mysqli_errno($handle);
echo '|';
echo mysqli_error($handle) === '' ? 'clean' : 'dirty';
