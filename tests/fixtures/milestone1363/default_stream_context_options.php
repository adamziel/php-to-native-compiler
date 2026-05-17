<?php
$path = "/tmp/phpc_milestone1363_default_stream_context.txt";
$seed = fopen($path, "w+");
fwrite($seed, "context-default");
fclose($seed);

$default = stream_context_get_default(array(
    "http" => array("method" => "GET"),
));
stream_context_set_option($default, "http", "header", "X-WP: one");
stream_context_set_option($default, array(
    "ssl" => array("verify_peer" => false),
    "http" => array("method" => "POST"),
));

$again = stream_context_get_default();
echo gettype($default);
echo "|";
echo $again === $default ? "same" : "different";
$null_default = stream_context_get_default(null);
echo ":";
echo $null_default === $default ? "null-same" : "null-different";
$options = stream_context_get_options($again);
echo "|";
echo $options["http"]["method"];
echo ":";
echo $options["http"]["header"];
echo ":";
echo $options["ssl"]["verify_peer"] ? "verify" : "skip";

$replacement = stream_context_set_default(array(
    "http" => array("timeout" => 7),
));
$default_options = stream_context_get_options(stream_context_get_default());
echo "|";
echo $replacement === $default ? "same-default" : "new-default";
echo ":";
echo $default_options["http"]["timeout"];
echo "|";
echo file_get_contents($path, false, $replacement);
