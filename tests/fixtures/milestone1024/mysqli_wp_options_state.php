<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')") ? "inserted" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
echo mysqli_insert_id($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
$row = mysqli_fetch_assoc($result);
echo $row["option_value"];
echo "|";
$missing = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'home' LIMIT 1");
echo mysqli_num_rows($missing);
