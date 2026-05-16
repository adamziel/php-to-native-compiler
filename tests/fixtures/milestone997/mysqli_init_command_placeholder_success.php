<?php
$handle = mysqli_init();
echo mysqli_options($handle, MYSQLI_INIT_COMMAND, "SET NAMES utf8mb4") ? "init-set" : "failed";
echo "|";
echo mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0) ? "connected" : "failed";
echo "|";
echo mysqli_field_count($handle);
echo "|";
echo mysqli_store_result($handle) === false ? "no-pending" : "pending";
