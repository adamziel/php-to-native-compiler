<?php
ob_start();
echo "outer:";
ob_start();
echo "inner";
$inner = ob_get_flush();
echo "|after-inner";
$outer = ob_get_flush();
echo "|inner=[" . $inner . "]";
echo "|outer=[" . $outer . "]";
echo "|level=" . ob_get_level();
