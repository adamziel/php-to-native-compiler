<?php
echo file_exists(__FILE__) ? "file" : "missing";
echo "|";
echo file_exists(__DIR__) ? "dir" : "missing";
echo "|";
echo file_exists(__DIR__ . "/missing-file.php") ? "exists" : "missing";

