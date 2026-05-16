<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
$id = 2;
mysqli_stmt_bind_param($stmt, "i", $id);
$id = 1;
echo mysqli_stmt_execute($stmt) ? "executed" : "not-executed";
$result = mysqli_stmt_get_result($stmt);
$row = mysqli_fetch_assoc($result);
echo "|", $row["ID"], ":", $row["post_title"];
