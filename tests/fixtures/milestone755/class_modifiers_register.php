<?php
abstract class Base {
    abstract protected function compute();
}

final class Leaf extends Base {
    public final function compute() {
        return "ok";
    }
}

readonly class Marker {}

$leaf = new Leaf();
echo $leaf->compute();
