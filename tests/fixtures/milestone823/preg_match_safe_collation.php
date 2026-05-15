<?php
$pattern = '/^(?:SHOW|DESCRIBE|DESC|EXPLAIN|CREATE)\s/i';

echo preg_match($pattern, "SHOW TABLES", $matches), "|";
echo $matches[0], "|";
echo preg_match($pattern, "select * from wp_options"), "|";
echo preg_match($pattern, "create\tTABLE wp_posts", $matches), "|";
echo $matches[0];
