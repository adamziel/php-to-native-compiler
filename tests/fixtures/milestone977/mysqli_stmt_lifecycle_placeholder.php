<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);

$stmt = mysqli_stmt_init($handle);
echo get_class($stmt);
echo "|";
echo mysqli_stmt_param_count($stmt);
echo "|";
echo mysqli_stmt_prepare($stmt, "SELECT option_value FROM wp_options WHERE option_name = ?")
    ? "prepared"
    : "failed";
echo "|";
echo mysqli_stmt_param_count($stmt);
echo "|";
echo mysqli_stmt_reset($stmt) ? "reset" : "failed";
echo "|";
echo mysqli_stmt_param_count($stmt);
echo "|";

$prepared = mysqli_prepare($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
echo get_class($prepared);
echo "|";
echo mysqli_stmt_param_count($prepared);
echo "|";
echo mysqli_stmt_close($prepared) ? "closed" : "open";
