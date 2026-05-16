<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo mysqli_stmt_execute($stmt) ? "executed" : "failed";
echo "|";
$result = mysqli_stmt_get_result($stmt);
echo get_class($result);
echo "|";
$row = mysqli_fetch_assoc($result);
echo $row["ID"];
echo "|";
echo $row["post_title"];
