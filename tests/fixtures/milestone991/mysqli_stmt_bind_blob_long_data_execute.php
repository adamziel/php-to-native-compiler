<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
$blob = "unused";
echo mysqli_stmt_bind_param($stmt, "b", $blob) ? "blob-bound" : "not-bound";
echo "|";
echo mysqli_stmt_send_long_data($stmt, 0, "1") ? "sent" : "send-failed";
echo "|";
echo mysqli_stmt_execute($stmt) ? "executed" : "not-executed";
$result = mysqli_stmt_get_result($stmt);
$row = mysqli_fetch_assoc($result);
echo "|", $row["ID"], ":", $row["post_title"];
