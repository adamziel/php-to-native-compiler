<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_stmt_execute($stmt);
echo mysqli_stmt_store_result($stmt) ? "stored" : "not-stored";
echo "|";
echo mysqli_stmt_num_rows($stmt);
mysqli_stmt_data_seek($stmt, 0);
echo "|seeked";
mysqli_stmt_free_result($stmt);
echo "|";
echo mysqli_stmt_num_rows($stmt);
