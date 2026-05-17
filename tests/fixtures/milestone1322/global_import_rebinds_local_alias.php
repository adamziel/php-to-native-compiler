<?php
$value = "root";

function rebind() {
    $local = ["slot" => "local"];
    $value =& $local["slot"];
    $value = "local-mutated";
    echo $value, "|";
    global $value;
    echo $value, "|";
    $value = "updated-root";
}

rebind();
echo $value;
