<?php
$stmt = mysqli_prepare(mysqli_init(), "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
echo mysqli_stmt_field_count($stmt);
echo "|";
$metadata = mysqli_stmt_result_metadata($stmt);
echo get_class($metadata);
echo "|";
echo mysqli_fetch_field_direct($metadata, 0)->name;
echo "|";
echo mysqli_fetch_field_direct($metadata, 1)->name;
echo "|";
mysqli_stmt_free_result($stmt);
echo "freed";
