<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);

$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$row = mysqli_fetch_array($result, MYSQLI_NUM);
echo $row[0], "\n";
echo $row[1], "\n";
echo isset($row["ID"]) ? "assoc" : "no-assoc", "\n";

$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$row = mysqli_fetch_array($result);
echo $row[0], "|", $row["ID"], "\n";
echo $row[1], "|", $row["post_title"], "\n";
echo mysqli_fetch_array($result) === false ? "no-row" : "row", "\n";

$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$row = mysqli_fetch_array($result, MYSQLI_BOTH);
echo $row[0], "|", $row["ID"], "\n";
echo $row[1], "|", $row["post_title"];
