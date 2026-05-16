<?php
$links = "mysqli_get_links_stats";
$stats = mysqli_get_links_stats();

echo function_exists($links) ? "yes" : "no";
echo "\n";
echo is_callable($links) ? "callable" : "missing";
echo "\n";
echo $stats["total"];
echo "\n";
echo $stats["active_plinks"];
echo "\n";
echo $stats["cached_plinks"];
echo "\n";

$dynamic = $links();
echo $dynamic["total"];
echo "\n";
echo $dynamic["active_plinks"];
echo "\n";
echo $dynamic["cached_plinks"];
