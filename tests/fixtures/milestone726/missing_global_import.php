<?php
function initialize_missing() {
    global $missing;
    var_dump($missing);
    $missing = ["status" => "ready"];
}
initialize_missing();
echo $missing["status"], "\n";
