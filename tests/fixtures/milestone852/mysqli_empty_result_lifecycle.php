<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
$result = mysqli_query($handle, "SELECT * FROM wp_posts WHERE 1 = 0");
echo get_class($result), "\n";
echo mysqli_num_fields($result), "\n";
echo mysqli_fetch_field($result) === false ? "no-field\n" : "field\n";
echo mysqli_fetch_object($result) === false ? "no-row\n" : "row\n";
echo mysqli_free_result($result) === null ? "freed\n" : "value\n";
echo mysqli_more_results($handle) ? "more\n" : "done\n";
echo mysqli_next_result($handle) ? "next" : "done";
