<?php
$handle = mysqli_init();
mysqli_real_connect($handle, 'localhost', 'user', 'pass', null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('blogdescription', 'value-kept', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'autoload-db', 'yes')");
echo mysqli_query($handle, "UPDATE wp_options SET autoload = 'auto-off' WHERE option_name = 'blogdescription'") ? 'updated' : 'failed';
echo ':', mysqli_affected_rows($handle), '|';
$row_result = mysqli_query($handle, "SELECT option_value, autoload FROM wp_options WHERE option_name = 'blogdescription' LIMIT 1");
$row = mysqli_fetch_assoc($row_result);
echo $row['option_value'], ':', $row['autoload'], '|';
$autoload = mysqli_query($handle, "SELECT option_id, option_name, option_value, autoload FROM wp_options WHERE autoload IN ( 'yes', 'on', 'auto-on', 'auto' )");
$autoload_first = mysqli_fetch_assoc($autoload);
echo mysqli_num_rows($autoload), ':';
echo $autoload_first['option_id'], ':', $autoload_first['option_name'], ':', $autoload_first['autoload'];
echo '|';
echo mysqli_query($handle, "UPDATE `wp_options` SET `autoload` = 'yes' WHERE `option_name` = 'missing'") ? 'missing-update' : 'failed';
echo ':', mysqli_affected_rows($handle);
