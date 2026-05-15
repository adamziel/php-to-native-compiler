<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$row = mysqli_fetch_array($result, MYSQLI_ASSOC);
echo MYSQLI_ASSOC, "|", MYSQLI_NUM, "|", MYSQLI_BOTH, "\n";
echo $row["ID"], "\n";
echo $row["post_title"], "\n";
echo mysqli_fetch_array($result, MYSQLI_ASSOC) === false ? "no-row" : "row";
