<?php
$handle = mysqli_init();
mysqli_real_connect($handle, 'localhost', 'user', 'pass', null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT option_name, option_value FROM wp_options WHERE option_name IN ('siteurl','home')");
$single = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
echo get_class($result);
echo '|';
echo mysqli_num_rows($result);
echo '|';
echo $single === false ? 'single-empty' : 'single-result';
echo '|';
mysqli_free_result($result);
echo mysqli_errno($handle);
echo '|';
echo mysqli_error($handle) === '' ? 'clean' : 'dirty';
