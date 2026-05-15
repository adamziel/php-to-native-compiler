<?php
$time = microtime(true);
echo is_float($time) ? 'float' : 'other';
echo '|';
echo $time > 0 ? 'positive' : 'non-positive';
