<?php
$target = __DIR__ . "/is_link_target.txt";
$link = __DIR__ . "/is_link_target_link.txt";

echo is_link($link) ? "link" : "not-link";
echo "|";
echo is_link($target) ? "link" : "not-link";
echo "|";
echo is_link(__DIR__ . "/missing-target.txt") ? "link" : "missing";
echo "|";
$call = "is_link";
echo function_exists($call) ? "exists" : "missing-function";
echo "|";
echo is_callable($call) ? "callable" : "not-callable";
echo "|";
echo $call($link) ? "repeat" : "changed";
