<?php
$cases = ["?", "#", "?q", "#f", "?#", "?#f", "?q#f", ""];
$separator = "";
foreach ($cases as $url) {
    echo $separator;
    $separator = "\n";
    echo $url === "" ? "<empty>" : $url, "|";
    $parts = parse_url($url);
    echo array_key_exists("path", $parts) ? "path" : "no-path";
    echo "|", parse_url($url, PHP_URL_PATH) === null ? "null" : parse_url($url, PHP_URL_PATH);
    echo "|", parse_url($url, PHP_URL_QUERY) === null ? "null" : parse_url($url, PHP_URL_QUERY);
    echo "|", parse_url($url, PHP_URL_FRAGMENT) === null ? "null" : parse_url($url, PHP_URL_FRAGMENT);
}
