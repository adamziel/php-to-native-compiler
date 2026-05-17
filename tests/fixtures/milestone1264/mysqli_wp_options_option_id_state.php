<?php
$handle = mysqli_init();
mysqli_real_connect($handle, 'localhost', 'user', 'pass', null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
$first = mysqli_query($handle, "SELECT option_id FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$first_row = mysqli_fetch_assoc($first);
echo $first_row['option_id'];
echo '|';
echo mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'duplicate', 'no')") ? 'duplicate' : 'rejected';
echo '|';
$again = mysqli_query($handle, "SELECT option_id FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$again_row = mysqli_fetch_assoc($again);
echo $again_row['option_id'];
echo '|';
$missing = mysqli_query($handle, "SELECT option_id FROM wp_options WHERE option_name = 'missing' LIMIT 1");
echo mysqli_num_rows($missing);
