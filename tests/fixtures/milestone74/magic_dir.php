<?php
echo __DIR__, "\n";
$dir = __DIR__;
echo $dir, "\n";

function default_dir($dir = __DIR__) {
    echo $dir, "\n";
}

const DECLARED_DIR = __DIR__;

default_dir();
echo DECLARED_DIR, "\n";
