<?php
$context = stream_context_create(
    array(
        "http" => array("method" => "GET", "header" => "X-WP: seed"),
        "ssl" => array("verify_peer" => true),
    ),
    array(
        "notification" => "first",
        "options" => array("http" => array("method" => "POST")),
        "ignored" => "value",
    )
);

$params = stream_context_get_params($context);
$options = stream_context_get_options($context);

echo gettype($context);
echo "|";
echo $params["notification"];
echo ":";
echo $params["options"]["http"]["method"];
echo ":";
echo $params["options"]["http"]["header"];
echo ":";
echo $options["http"]["method"];
echo ":";
echo $params["options"]["ssl"]["verify_peer"] ? "verify" : "skip";
echo ":";
echo isset($params["ignored"]) ? "kept" : "ignored";

echo "|";
echo stream_context_set_params($context, array(
    "notification" => "second",
    "options" => array("ssl" => array("verify_peer" => false)),
)) ? "set" : "failed";

$updated = stream_context_get_params($context);
echo "|";
echo $updated["notification"];
echo ":";
echo $updated["options"]["http"]["method"];
echo ":";
echo $updated["options"]["ssl"]["verify_peer"] ? "verify" : "skip";

stream_context_set_params($context, array(
    "options" => array("http" => array("header" => "X-WP: options-only")),
));
$preserved = stream_context_get_params($context);
echo "|";
echo $preserved["notification"];
echo ":";
echo $preserved["options"]["http"]["header"];
