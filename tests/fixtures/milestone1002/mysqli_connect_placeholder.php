<?php
$handle = mysqli_connect("localhost", "user", "password", "wordpress", 3306, null);
echo get_class($handle);
echo "|";
echo mysqli_get_server_info($handle);
echo "|";
$result = mysqli_query($handle, "SELECT ID, post_title FROM wp_posts WHERE ID = 1");
$row = mysqli_fetch_assoc($result);
echo $row["ID"], ":", $row["post_title"];
