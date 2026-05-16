<?php
abstract class Base {
    abstract protected function compute();
}

final class Leaf extends Base {
    public final function compute() {
        return "ok";
    }
}

$leaf = new Leaf();
echo $leaf->compute();
