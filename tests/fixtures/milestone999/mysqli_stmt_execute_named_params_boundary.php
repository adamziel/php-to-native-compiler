<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = ?");
mysqli_stmt_execute($stmt, array("id" => 1));

