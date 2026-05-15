<?php
echo is_readable(__FILE__) ? "file" : "missing";
echo '|';
echo is_readable(__DIR__) ? "dir" : "missing";
echo '|';
echo is_readable(__DIR__ . '/missing-file.php') ? "readable" : "missing";
