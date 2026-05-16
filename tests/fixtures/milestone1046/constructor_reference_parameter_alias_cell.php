<?php
function observe_value() {
    global $value;
    echo "seen=", $value, "|";
}

class MutatesParameter {
    public function __construct(&$param) {
        $param = 2;
        observe_value();
        $param = 3;
    }
}

class DetachesParameter {
    public function __construct(&$param) {
        unset($param);
        $param = 9;
        echo "local=", $param, "|";
    }
}

$value = 1;
$first = new MutatesParameter($value);
echo "final=", $value, "\n";

$second = new DetachesParameter($value);
echo "caller=", $value;
