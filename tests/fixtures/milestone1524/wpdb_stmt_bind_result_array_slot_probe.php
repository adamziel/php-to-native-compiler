<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
mysqli_stmt_execute($stmt, array("1"));
mysqli_stmt_store_result($stmt);

$row = array();
$id_key = "ID";

echo mysqli_stmt_bind_result($stmt, $row[$id_key], $row["post_title"]) ? "bound" : "failed";
echo "|";
echo mysqli_stmt_fetch($stmt) ? $row["ID"] . ":" . $row["post_title"] : "no-row";
echo "|";
$id_key = "changed";
echo mysqli_stmt_fetch($stmt) === null ? "done" : "again";
echo "|";
echo array_key_exists("changed", $row) ? "changed" : "stable";
