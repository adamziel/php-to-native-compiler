<?php
echo __FILE__, "\n";
$file = __FILE__;
echo $file, "\n";

function default_file($file = __FILE__) {
    echo $file, "\n";
}

const DECLARED_FILE = __FILE__;

default_file();
echo DECLARED_FILE, "\n";
