<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
echo function_exists("mysqli_execute") ? "exists" : "missing";
echo "|";
echo is_callable("mysqli_execute") ? "callable" : "missing";
echo "|";
echo mysqli_execute($stmt, array(1)) ? "executed" : "failed";
$result = mysqli_stmt_get_result($stmt);
$row = mysqli_fetch_assoc($result);
echo "|";
echo $row["ID"], ":", $row["post_title"];
