<?php
ob_start();
echo "before";
echo "|level=" . ob_get_level();
ob_start();
echo "inner";
$inner = ob_get_clean();
echo "|inner=" . $inner;
$outer = ob_get_clean();
echo "captured=[" . $outer . "]";
echo "|level=" . ob_get_level();
