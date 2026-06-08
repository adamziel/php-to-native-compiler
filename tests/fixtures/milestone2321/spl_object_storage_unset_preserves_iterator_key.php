<?php
function named_object($name) {
    $object = new stdClass();
    $object->name = $name;
    return $object;
}

$syntax = new SplObjectStorage();
$a = named_object("a");
$b = named_object("b");
$c = named_object("c");
$syntax[$a] = "a";
$syntax[$b] = "b";
$syntax[$c] = "c";
$syntax->next();
unset($syntax[$a]);
echo "syntax:", $syntax->key(), ":", $syntax->current()->name, "\n";
$syntax->next();
echo "syntax:", $syntax->key(), ":", $syntax->current()->name, "\n";
$syntax->next();
echo "syntax:", $syntax->key(), ":", $syntax->valid() ? "valid" : "invalid", "\n";

$method = new SplObjectStorage();
$ma = named_object("a");
$mb = named_object("b");
$mc = named_object("c");
$method[$ma] = "a";
$method[$mb] = "b";
$method[$mc] = "c";
$method->next();
$method->detach($ma);
echo "method:", $method->key(), ":", $method->current()->name, "\n";
$method->next();
echo "method:", $method->key(), ":", $method->current()->name;
