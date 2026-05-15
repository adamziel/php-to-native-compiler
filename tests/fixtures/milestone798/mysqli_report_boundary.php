<?php
$call = 'mysqli_report';
echo function_exists($call) ? 'yes' : 'no';
echo '|';
echo is_callable($call) ? 'callable' : 'missing';
echo '|';
echo MYSQLI_REPORT_OFF;
echo '|';
echo mysqli_report(MYSQLI_REPORT_OFF) ? 'off' : 'fail';
