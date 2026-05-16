<?php
class Bag {
    public $name = "declared";
    public $log = [];

    public function __set($property, $value) {
        echo "set:$property=$value\n";
        $this->log[$property] = $value;
        return "ignored";
    }

    public function __get($property) {
        return $this->log[$property];
    }
}

$bag = new Bag();
$bag->name = "direct";
echo $bag->name, "\n";
$result = ($bag->title = "Hello");
echo "result:$result\n";
echo $bag->title;
