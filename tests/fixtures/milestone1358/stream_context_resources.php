<?php
$path = "/tmp/phpc_milestone1358_stream_context.txt";
$seed = fopen($path, "w");
fwrite($seed, "context-cache");
fclose($seed);

$context = stream_context_create(array(
    "http" => array("method" => "POST", "header" => "X-WP: plugin"),
    "ssl" => array("verify_peer" => false),
));
$options = stream_context_get_options($context);

echo gettype($context);
echo "|";
echo $options["http"]["method"];
echo ":";
echo $options["http"]["header"];
echo ":";
echo ($options["ssl"]["verify_peer"] ? "verify" : "skip");

echo "|";
echo file_get_contents($path, false, $context);
echo ":";
echo file_get_contents($path, false, null);

$stream = fopen($path, "r", false, $context);
$meta = stream_get_meta_data($stream);
echo "|";
echo $meta["wrapper_type"];
echo ":";
echo $meta["stream_type"];
echo ":";
echo stream_get_contents($stream);
fclose($stream);
