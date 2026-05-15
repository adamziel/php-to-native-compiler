<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$row = mysqli_fetch_assoc($result);
echo $row["ID"], "\n";
echo $row["post_title"], "\n";
echo mysqli_fetch_assoc($result) === false ? "no-row" : "row";
