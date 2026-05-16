<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_execute_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = ?", array(1));
$row = mysqli_fetch_assoc($result);
echo $row["ID"], ":", $row["post_title"];
echo "|";
$empty = mysqli_execute_query($handle, "SELECT option_value FROM wp_options WHERE option_name = ?", array("siteurl"));
echo mysqli_num_rows($empty), ":", mysqli_num_fields($empty);
echo "|";
echo mysqli_execute_query($handle, "SET SESSION sql_mode=''") ? "no-result" : "failed";

