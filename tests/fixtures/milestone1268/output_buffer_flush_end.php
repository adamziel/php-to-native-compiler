<?php
ob_start();
echo "outer:";
ob_start();
echo "inner";
ob_flush();
echo "|after-flush";
$ended = ob_end_flush();
$peek = ob_get_contents();
echo "|after-end";
$final = ob_get_clean();

ob_start();
echo "discard";
$cleaned = ob_clean();
echo "kept";
$discarded = ob_end_clean();

echo "final=[" . $final . "]";
echo "|peek=[" . $peek . "]";
echo "|ended=" . ($ended ? "true" : "false");
echo "|cleaned=" . ($cleaned ? "true" : "false");
echo "|discarded=" . ($discarded ? "true" : "false");
