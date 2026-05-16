<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
echo mysqli_stmt_execute($stmt, array(1)) ? "array-executed" : "not-executed";
$result = mysqli_stmt_get_result($stmt);
$row = mysqli_fetch_assoc($result);
echo "|", $row["ID"], ":", $row["post_title"];
$stmt2 = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
echo "|";
echo call_user_func("mysqli_stmt_execute", $stmt2, array(1)) ? "callback-array-executed" : "not-executed";
$result2 = mysqli_stmt_get_result($stmt2);
$row2 = mysqli_fetch_assoc($result2);
echo "|", $row2["ID"], ":", $row2["post_title"];
