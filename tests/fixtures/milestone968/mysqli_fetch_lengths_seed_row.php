<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo mysqli_fetch_lengths($result) === false ? "no-lengths" : "lengths";
$row = mysqli_fetch_assoc($result);
$lengths = mysqli_fetch_lengths($result);
echo "|";
echo $lengths[0];
echo ",";
echo $lengths[1];
