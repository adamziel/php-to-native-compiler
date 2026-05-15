<?php
$call = 'mysqli_connect';
echo function_exists($call) ? 'yes' : 'no';
echo '|';
echo is_callable($call) ? 'callable' : 'missing';
