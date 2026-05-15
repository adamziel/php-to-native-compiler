<?php
$handle = mysqli_init();
echo mysqli_affected_rows($handle), "\n";
echo mysqli_insert_id($handle), "\n";

mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_query($handle, "SET NAMES 'utf8mb4' COLLATE 'utf8mb4_unicode_520_ci'");
echo mysqli_affected_rows($handle), "\n";
echo mysqli_insert_id($handle);
