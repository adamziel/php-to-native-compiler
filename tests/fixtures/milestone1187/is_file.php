<?php
echo is_file(__FILE__) ? "file" : "missing";
echo "|";
echo is_file(__DIR__) ? "dir" : "not-file";
echo "|";
echo is_file(__DIR__ . "/missing-file.php") ? "file" : "missing";

