<?php
$thread = "mysqli_thread_id";
$kill = "mysqli_kill";
$dbh = mysqli_init();
mysqli_options($dbh, MYSQLI_OPT_INT_AND_FLOAT_NATIVE, true);
mysqli_real_connect($dbh, "localhost", "user", "pass", null, 3306, null, 0);

echo function_exists($kill) ? "yes" : "no";
echo "\n";
echo is_callable($kill) ? "callable" : "missing";
echo "\n";
$thread_id = $thread($dbh);
echo $thread_id;
echo "\n";
echo mysqli_kill($dbh, $thread_id) ? "killed-placeholder" : "kill-failed";
echo "\n";
echo $kill($dbh, 99) ? "unexpected-thread" : "no-thread";
echo "\n";
echo mysqli_ping($dbh) ? "still-open" : "closed";
