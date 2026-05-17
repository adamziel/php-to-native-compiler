<?php
ob_start();
echo "buffer";
$before = headers_sent($file, $line);
echo "|" . ($before ? "sent" : "open") . ":" . $file . ":" . $line;
ob_flush();
$after = headers_sent($file, $line);
echo "|" . ($after ? "sent" : "open") . ":" . basename($file) . ":" . $line;
