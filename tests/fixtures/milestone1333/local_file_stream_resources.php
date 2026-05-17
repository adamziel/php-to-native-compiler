<?php
$path = "/tmp/phpc_milestone1333_file_stream_cache.txt";
$stream = fopen($path, "w+");
fwrite($stream, "wp-cache");
fwrite($stream, "-data", 5);
rewind($stream);
$prefix = fread($stream, 2);
$rest = stream_get_contents($stream);
fclose($stream);

$append = fopen($path, "a+");
fwrite($append, "-tail");
rewind($append);

echo gettype($append) . "|" . $prefix . "|" . $rest . "|" . stream_get_contents($append);
echo "|" . (fclose($append) ? "closed" : "open");
