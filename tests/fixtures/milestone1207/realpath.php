<?php
$path = __DIR__ . "/realpath_target.txt";
$resolved = realpath($path);

echo is_string($resolved) ? "string" : "not-string";
echo "|";
echo str_ends_with($resolved, "/tests/fixtures/milestone1207/realpath_target.txt") ? "suffix" : "bad-suffix";
echo "|";
echo realpath(__DIR__ . "/missing-target.txt") === false ? "missing" : "unexpected";
echo "|";
$call = "realpath";
echo $call($path) === $resolved ? "repeat" : "changed";
