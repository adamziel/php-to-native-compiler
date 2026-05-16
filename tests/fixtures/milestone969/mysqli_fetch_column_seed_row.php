<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo mysqli_fetch_column($result);
echo "|";
echo mysqli_fetch_column($result) === false ? "false" : "value";
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo "|";
echo mysqli_fetch_column($result, 1);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo "|";
echo mysqli_fetch_column($result, 99) === null ? "null" : "value";
