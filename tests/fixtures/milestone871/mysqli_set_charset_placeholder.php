<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo mysqli_set_charset($handle, "utf8mb4") ? "set" : "failed", "\n";
$call = "mysqli_set_charset";
echo $call($handle, "UTF8MB4") ? "dynamic" : "failed";
