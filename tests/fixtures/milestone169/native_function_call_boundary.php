<?php
echo label("user"), "\n";
function label($value) {
    return $value . "!";
}
$call = "label";
echo $call("dynamic"), "\n";
$builtin = "strlen";
echo $builtin("callable");
