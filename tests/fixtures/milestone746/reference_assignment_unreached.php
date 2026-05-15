<?php
if (false) {
    $a = 1;
    $b =& $a;
}

function parse_args($args) {
    $parsed_args =& $args;
    return $parsed_args;
}

echo "ok";
