<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_update_plugins', 'plugin-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_site_transient_update_plugins', 'site-payload', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('_transient_timeout_update_plugins', '123', 'no')");
mysqli_query($handle, "INSERT INTO wp_options (option_name, option_value, autoload) VALUES ('siteurl', 'https://example.test', 'yes')");

$rows = mysqli_execute_query($handle, "SELECT option_name, option_value FROM wp_options WHERE (option_name LIKE ? OR option_name LIKE ?) ORDER BY option_name", array("\\_transient\\_%", "\\_site\\_transient\\_%"));
echo "execute=", mysqli_num_rows($rows), ":";
while ($row = mysqli_fetch_assoc($rows)) {
    echo $row["option_name"], "=", $row["option_value"], ";";
}

$stmt = mysqli_prepare($handle, "SELECT `option_name`, `option_value`, `autoload` FROM `wp_options` WHERE (`option_name` LIKE ? OR `option_name` LIKE ?) ORDER BY `option_name` ASC");
$timeout = "\\_transient\\_timeout\\_%";
$site = "\\_site\\_transient\\_%";
mysqli_stmt_bind_param($stmt, "ss", $timeout, $site);
mysqli_stmt_execute($stmt);
$bound = mysqli_stmt_get_result($stmt);
echo "|bound=", mysqli_num_fields($bound), ":", mysqli_num_rows($bound), ":";
while ($row = mysqli_fetch_assoc($bound)) {
    echo $row["option_name"], "=", $row["option_value"], ":", $row["autoload"], ";";
}
