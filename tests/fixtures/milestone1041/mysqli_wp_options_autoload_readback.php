<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);

mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
mysqli_query($handle, "REPLACE INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://replaced.test', 'no')");

$result = mysqli_query($handle, "SELECT autoload FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["autoload"];
echo "|";
echo mysqli_num_rows($result);
echo "|";
echo mysqli_affected_rows($handle);
echo "|";

$missing = mysqli_query($handle, "SELECT autoload FROM wp_options WHERE option_name = 'home' LIMIT 1");
echo mysqli_num_rows($missing);
echo "|";

mysqli_query($handle, "INSERT INTO `wp_options` (`option_name`, `option_value`, `autoload`) VALUES ('home', 'https://home.test', 'auto-on')");
$home = mysqli_query($handle, "SELECT `autoload` FROM `wp_options` WHERE `option_name` = 'home' LIMIT 1");
$home_row = mysqli_fetch_assoc($home);
echo $home_row["autoload"];
