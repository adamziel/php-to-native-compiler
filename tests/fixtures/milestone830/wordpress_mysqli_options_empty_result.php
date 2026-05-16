<?php
$handle = mysqli_init();
mysqli_real_connect($handle, 'localhost', 'user', 'pass', null, 3306, null, 0);
$autoload = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options WHERE autoload IN ( 'yes', 'on', 'auto-on', 'auto' )");
$fallback = mysqli_query($handle, 'SELECT option_name, option_value FROM wp_options');
echo get_class($autoload);
echo '|';
echo mysqli_num_rows($autoload);
echo '|';
echo $fallback === false ? 'fallback-empty' : 'fallback-result';
echo '|';
mysqli_free_result($autoload);
echo mysqli_errno($handle);
echo '|';
echo mysqli_error($handle) === '' ? 'clean' : 'dirty';
