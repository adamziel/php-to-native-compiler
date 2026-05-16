<?php
abstract class Base {
    abstract protected function compute();
}

abstract class Mid extends Base {}

class Leaf extends Mid {
    public function compute() {
        return "leaf";
    }
}

$leaf = new Leaf();
echo $leaf->compute();
