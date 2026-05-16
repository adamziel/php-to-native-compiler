<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
mysqli_stmt_execute($stmt);
echo mysqli_stmt_more_results($stmt) ? "more" : "no-more";
echo "|";
echo mysqli_stmt_next_result($stmt) ? "next" : "no-next";
