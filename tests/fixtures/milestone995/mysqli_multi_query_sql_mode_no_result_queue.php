<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo mysqli_multi_query($handle, "SELECT @@SESSION.sql_mode; SELECT ID, post_title FROM wp_posts WHERE ID = 1") ? "queued" : "failed";
echo "|";
echo mysqli_field_count($handle);
echo "|";
echo mysqli_store_result($handle) === false ? "no-result" : "result";
echo "|";
echo mysqli_next_result($handle) ? "next" : "blocked";
$result = mysqli_store_result($handle);
$row = mysqli_fetch_assoc($result);
echo "|";
echo $row["ID"], ":", $row["post_title"];
