<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
$id = 2;
mysqli_stmt_bind_param($stmt, "i", $id);
$id = 1;
echo call_user_func("mysqli_stmt_execute", $stmt) ? "call-user-func" : "not-executed";
$result = mysqli_stmt_get_result($stmt);
$row = mysqli_fetch_assoc($result);
echo "|", $row["ID"], ":", $row["post_title"];
$stmt2 = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
$id2 = 2;
mysqli_stmt_bind_param($stmt2, "i", $id2);
$id2 = 1;
echo "|";
echo call_user_func_array("mysqli_stmt_execute", array($stmt2)) ? "call-user-func-array" : "not-executed";
$result2 = mysqli_stmt_get_result($stmt2);
$row2 = mysqli_fetch_assoc($result2);
echo "|", $row2["ID"], ":", $row2["post_title"];
