<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$rows = mysqli_fetch_all($result);
echo $rows[0][0];
echo "|";
echo $rows[0][1];
echo "|";
echo isset($rows[0]["ID"]) ? "assoc" : "no-assoc";
echo "|";
echo mysqli_fetch_assoc($result) === false ? "no-row" : "row";
$lengths = mysqli_fetch_lengths($result);
echo "|";
echo $lengths[0];
echo ",";
echo $lengths[1];
