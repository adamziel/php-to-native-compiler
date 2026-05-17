<?php
$tmp = "/tmp/phpc-milestone1378-upload-source.txt";
$dest = "/tmp/phpc-milestone1378-upload-dest.txt";

echo is_uploaded_file($tmp) ? "uploaded" : "plain";
echo "|";
echo move_uploaded_file($tmp, $dest) ? "moved" : "stayed";
