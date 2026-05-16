<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
echo mysqli_query($handle, "SET SESSION sql_mode=''") ? "query-ok" : "query-failed";
echo "|";
echo mysqli_real_query($handle, "SET SESSION sql_mode='NO_ENGINE_SUBSTITUTION'") ? "real-ok" : "real-failed";
echo "|";
echo mysqli_field_count($handle);
echo "|";
echo mysqli_store_result($handle) === false ? "no-result" : "result";

