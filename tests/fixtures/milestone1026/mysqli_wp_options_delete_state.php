<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");
echo mysqli_query($handle, "DELETE FROM wp_options WHERE option_name = 'siteurl'") ? "deleted" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
echo "|";
$result = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'siteurl' LIMIT 1");
echo mysqli_num_rows($result);
echo "|";
echo mysqli_query($handle, "DELETE FROM wp_options WHERE option_name = 'home'") ? "missing-delete" : "failed";
echo "|";
echo mysqli_affected_rows($handle);
