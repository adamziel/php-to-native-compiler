<?php
// alpha
// beta
// gamma
$file = new SplFileObject(__FILE__);
echo $file->current();
$file->seek(2);
echo $file->key(), ":", $file->current();
$file->next();
echo $file->key(), ":", $file->current();
$file->seek(99);
var_dump($file->valid());
$file->rewind();
foreach ($file as $key => $line) {
    if ($key > 1) {
        break;
    }
    echo $key, "=", $line;
}
try {
    $file->seek(-1);
} catch (ValueError $e) {
    echo "caught:", $e->getMessage();
}
