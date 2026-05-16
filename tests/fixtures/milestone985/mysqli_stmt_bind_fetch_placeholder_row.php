<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_stmt_execute($stmt);
mysqli_stmt_store_result($stmt);
$id = null;
$title = null;
echo mysqli_stmt_bind_result($stmt, $id, $title) ? "bound" : "not-bound";
echo "|";
echo mysqli_stmt_fetch($stmt) ? $id . ":" . $title : "no-row";
echo "|";
echo mysqli_stmt_fetch($stmt) ? "again" : "done";
