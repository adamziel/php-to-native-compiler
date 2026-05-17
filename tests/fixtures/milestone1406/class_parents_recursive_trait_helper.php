<?php
namespace App;

trait RootTrait {}
trait MidTrait {}
trait LeafTrait {}

class Root {
    use RootTrait;
}

class Mid extends Root {
    use MidTrait;
}

class Leaf extends Mid {
    use LeafTrait;
}

function class_uses_recursive_probe($class) {
    $traits = array();
    foreach (class_parents($class, false) as $parent) {
        foreach (class_uses($parent, false) as $trait) {
            $traits[$trait] = $trait;
        }
    }
    foreach (class_uses($class, false) as $trait) {
        $traits[$trait] = $trait;
    }
    return $traits;
}

$leaf = new Leaf();
print_r(class_parents($leaf));
print_r(class_parents("App\\Leaf", false));
print_r(class_uses_recursive_probe($leaf));
echo "helper-done";
