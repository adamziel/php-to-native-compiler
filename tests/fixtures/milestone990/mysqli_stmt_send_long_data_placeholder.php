<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT option_value FROM wp_options WHERE option_name = ?");
echo mysqli_stmt_send_long_data($stmt, 0, "blob") ? "sent" : "failed";
echo "|";
$send = "mysqli_stmt_send_long_data";
echo $send($stmt, 0, "-chunk") ? "sent-dynamic" : "failed";
echo "|";
echo mysqli_stmt_reset($stmt) ? "reset" : "failed";
