<?php
class Bag {
    public $name;
    public $log = [];

    public function __unset($property) {
        echo "unset:$property\n";
        $this->log[$property] = "gone";
    }

    public function __get($property) {
        return $this->log[$property];
    }
}

$bag = new Bag();
$bag->name = "Ada";
unset($bag->name);
echo isset($bag->name) ? "name:set\n" : "name:unset\n";
unset($bag->title);
echo $bag->title;
