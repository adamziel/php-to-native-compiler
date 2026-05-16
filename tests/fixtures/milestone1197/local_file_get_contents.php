<?php
$path = __DIR__ . "/local_read_payload.txt";
$contents = file_get_contents($path);
echo "[", $contents, "]\n";

$call = "file_get_contents";
echo $call($path) === $contents ? "repeat" : "different";
