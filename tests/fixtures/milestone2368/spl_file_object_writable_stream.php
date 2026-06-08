<?php
$file = __DIR__ . "/spl_file_object_writable_stream.tmp";
$object = new SplFileObject($file, "w+");
$object->setCsvControl("|", "'", "");
var_dump($object->fputcsv(array("a|b", "c")));
var_dump($object->ftell(), $object->eof());
var_dump($object->fwrite("tail", 2));
var_dump($object->fflush());
echo file_get_contents($file);
unlink($file);
