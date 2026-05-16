<?php
$thread_safe = "mysqli_thread_safe";

echo function_exists($thread_safe) ? "yes" : "no";
echo "\n";
echo is_callable($thread_safe) ? "callable" : "missing";
echo "\n";
echo mysqli_thread_safe() ? "thread-safe" : "not-safe";
echo "\n";
echo $thread_safe() ? "dynamic" : "not-safe";
