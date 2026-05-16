<?php
$cwd = getcwd();
echo is_string($cwd) ? "string" : "not-string";
echo "|";
echo strlen($cwd) > 0 ? "non-empty" : "empty";
echo "|";
echo is_dir($cwd) ? "dir" : "missing";
echo "|";
$call = "getcwd";
echo $call() === $cwd ? "repeat" : "changed";
