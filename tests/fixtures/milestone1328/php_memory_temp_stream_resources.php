<?php
$memory = fopen("php://memory", "w+");
fwrite($memory, "wp");
fwrite($memory, "-temp");
rewind($memory);
$prefix = fread($memory, 2);
$rest = stream_get_contents($memory);

$temp = fopen("php://temp", "w+b");
fwrite($temp, "request-body-cache");
rewind($temp);

echo $prefix . "|" . $rest . "|" . stream_get_contents($temp);
echo "|" . (fclose($memory) ? "closed" : "open");
