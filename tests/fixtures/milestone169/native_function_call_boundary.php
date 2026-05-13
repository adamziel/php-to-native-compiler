<?php
echo strlen("abc"), "\n";
function label($value) {
    return $value . "!";
}
echo label("user"), "\n";
$call = "label";
echo $call("dynamic"), "\n";
$builtin = "strlen";
echo $builtin("callable");
