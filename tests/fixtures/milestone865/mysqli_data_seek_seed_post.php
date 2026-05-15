<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$row = mysqli_fetch_assoc($result);
echo $row["post_title"], "\n";
echo mysqli_fetch_assoc($result) === false ? "no-row" : "row", "\n";
echo mysqli_data_seek($result, 0) ? "seek" : "no-seek", "\n";
$row = mysqli_fetch_row($result);
echo $row[0], "\n";
echo $row[1], "\n";
echo mysqli_data_seek($result, 1) ? "seek" : "no-seek";
