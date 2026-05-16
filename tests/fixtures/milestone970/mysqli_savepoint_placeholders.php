<?php
$handle = mysqli_init();
mysqli_real_connect($handle, "localhost", "user", "pass", null, 3306, null, 0);
mysqli_begin_transaction($handle);
echo mysqli_savepoint($handle, "wp") ? "savepoint" : "failed";
echo "|";
echo mysqli_release_savepoint($handle, "wp") ? "release" : "failed";
echo "|";
echo mysqli_commit($handle) ? "commit" : "failed";
