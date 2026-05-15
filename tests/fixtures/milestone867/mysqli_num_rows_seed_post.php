<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);

$empty = mysqli_query($handle, "SELECT * FROM wp_posts WHERE 1 = 0");
echo mysqli_num_rows($empty), "\n";
echo mysqli_fetch_assoc($empty) === false ? "empty" : "row", "\n";

$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo mysqli_num_rows($result), "\n";
$row = mysqli_fetch_assoc($result);
echo $row["ID"], "|", $row["post_title"], "\n";
echo mysqli_num_rows($result), "\n";
echo mysqli_fetch_assoc($result) === false ? "no-row" : "row";
