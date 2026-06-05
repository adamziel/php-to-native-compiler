<?php
$count = 0;
echo str_ireplace("wp", "php", "WP wp", $count), "|", $count, "\n";
$count = 0;
echo str_ireplace(["%0d", "%0a"], "", "%0%0DDD%0A", $count), "|", $count, "\n";
$call = "str_ireplace";
echo $call("A", "b", "a-A");
