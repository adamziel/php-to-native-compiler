<?php
$stats_call = "mysqli_get_client_stats";
$stats = mysqli_get_client_stats();

echo function_exists($stats_call) ? "yes" : "no";
echo "\n";
echo is_callable($stats_call) ? "callable" : "missing";
echo "\n";
echo $stats["bytes_sent"];
echo "\n";
echo $stats["bytes_received"];
echo "\n";
echo $stats["packets_sent"];
echo "\n";
echo $stats["packets_received"];
echo "\n";
echo $stats["protocol_overhead_in"];
echo "\n";
echo $stats["protocol_overhead_out"];
echo "\n";
echo $stats["connect_success"];
echo "\n";
echo $stats["active_connections"];
echo "\n";
$dynamic = $stats_call();
echo $dynamic["bytes_sent"];
