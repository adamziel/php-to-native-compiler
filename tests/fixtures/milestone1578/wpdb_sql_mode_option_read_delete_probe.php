<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);

mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('mode\\\\_target', 'with-backslash', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('plain_target', 'without-backslash', 'no')");

$default = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'mode\\_target' LIMIT 1");
echo "default=", mysqli_num_rows($default);

mysqli_query($handle, "SET SESSION sql_mode='NO_BACKSLASH_ESCAPES'");

$mode = mysqli_query($handle, "SELECT option_name, option_value, autoload FROM wp_options WHERE option_name = 'mode\\_target' LIMIT 1");
$mode_row = mysqli_fetch_assoc($mode);
echo "|mode=", mysqli_num_rows($mode), ":", $mode_row["option_name"], "=", $mode_row["option_value"], ":", $mode_row["autoload"];

echo "|delete=", mysqli_query($handle, "DELETE FROM wp_options WHERE option_name IN ('mode\\_target')") ? "ok" : "failed";
echo ":", mysqli_affected_rows($handle);

$remaining = mysqli_query($handle, "SELECT option_value FROM wp_options WHERE option_name = 'plain_target' LIMIT 1");
$remaining_row = mysqli_fetch_assoc($remaining);
echo "|left=", mysqli_num_rows($remaining), ":", $remaining_row["option_value"];
