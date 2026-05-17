<?php
$initial = ob_get_status();
ob_start();
echo "outer";
$outer = ob_get_status();
ob_start();
echo "xy";
$inner = ob_get_status();
$full = ob_get_status(true);
$inner_capture = ob_get_clean();
$after_inner = ob_get_status();
$outer_capture = ob_get_clean();
echo "initial=" . count($initial);
echo "|outer=" . $outer["name"] . ":" . $outer["level"] . ":" . $outer["buffer_used"] . ":" . $outer["chunk_size"] . ":" . $outer["buffer_size"];
echo "|inner=" . $inner["name"] . ":" . $inner["level"] . ":" . $inner["buffer_used"];
echo "|full=" . count($full) . ":" . $full[0]["level"] . ":" . $full[0]["buffer_used"] . ":" . $full[1]["level"] . ":" . $full[1]["buffer_used"];
echo "|after-inner=" . $after_inner["level"] . ":" . $after_inner["buffer_used"];
echo "|captures=" . $outer_capture . "/" . $inner_capture;
echo "|final=" . count(ob_get_status(true));
