<?php
$path = __DIR__ . "/spl_file_object_flags_csv_data.csv";
file_put_contents($path, "'green apples'|10\n'yellow bananas'|20\n");

$file = new SplFileObject($path);
$file->setFlags(SplFileObject::DROP_NEW_LINE);
var_dump($file->getFlags());
echo $file->current(), "\n";
var_dump($file->getCsvControl());

$file->setFlags(SplFileObject::READ_CSV);
$file->setCsvControl("|", "'", "");
var_dump($file->getFlags());
var_dump($file->getCsvControl());

foreach ($file as $row) {
    echo $row[0], "=", $row[1], "\n";
}

$file->rewind();
var_dump($file->fgetcsv());

try {
    $file->setCsvControl("||");
} catch (ValueError $e) {
    echo $e->getMessage();
}

unlink($path);
