<?php
ob_start();
echo "alpha";
$first = ob_get_contents();
echo "|beta";
$second = ob_get_contents();
$clean = ob_get_clean();
echo "first=[" . $first . "]";
echo "|second=[" . $second . "]";
echo "|clean=[" . $clean . "]";
echo "|empty=" . (ob_get_contents() === false ? "false" : "not-false");
