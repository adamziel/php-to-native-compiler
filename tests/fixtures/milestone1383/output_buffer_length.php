<?php
echo ob_get_length() === false ? "initial=false" : "initial=true";
ob_start();
echo "abc";
echo "|len=" . ob_get_length();
ob_start();
echo "xy";
echo "|inner-len=" . ob_get_length();
$inner = ob_get_clean();
echo "|after-inner-len=" . ob_get_length();
$outer = ob_get_clean();
echo "outer=[" . $outer . "]";
echo "|inner=[" . $inner . "]";
echo "|final=" . (ob_get_length() === false ? "false" : "true");
