<?php
function parse_args($args, $defaults) {
    if (is_array($args)) {
        $parsed_args =& $args;
    }
    return array_merge($defaults, $parsed_args);
}
$parsed = parse_args(["name" => "Ada"], ["role" => "admin"]);
echo $parsed["role"], "|", $parsed["name"];
