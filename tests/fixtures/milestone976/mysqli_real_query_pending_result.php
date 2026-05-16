<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);

echo mysqli_real_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1")
    ? "queued"
    : "failed";
echo "|";
echo mysqli_field_count($handle);
echo "|";

$result = mysqli_store_result($handle);
$row = mysqli_fetch_array($result, MYSQLI_ASSOC);
echo get_class($result), ":", $row["ID"], ":", $row["post_title"];
echo "|";
echo mysqli_store_result($handle) === false ? "drained" : "pending";

$other = mysqli_init();
mysqli_real_connect($other, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_real_query($other, "SELECT * FROM wp_posts WHERE 1 = 0");
$empty = mysqli_use_result($other);
echo "|";
echo get_class($empty), ":", mysqli_num_rows($empty), ":", mysqli_num_fields($empty);
