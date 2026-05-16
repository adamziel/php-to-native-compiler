<?php
$poll = "mysqli_poll";
echo function_exists($poll) ? "yes" : "no";
echo "\n";
echo is_callable($poll) ? "callable" : "missing";
echo "\n";
echo MYSQLI_ASYNC;
echo "\n";

$read = [];
$error = [];
$reject = [];
mysqli_poll($read, $error, $reject, 0);
