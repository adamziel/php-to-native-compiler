<?php
class Peer {}

class Box {
    public int $id;
    public ?string $name;
    public float $ratio;
    public Peer $peer;
    public static bool $ready;
    public static ?string $label;
}

function yn($value) {
    return $value ? "1" : "0";
}

function line($label, $property) {
    echo $label, "|", yn($property->hasDefaultValue()), "|", yn($property->hasType()), "|", ($property->getDefaultValue() === null ? "null" : "value"), "\n";
}

$box = new Box();
echo "initial|", yn(isset($box->id)), yn(empty($box->id)), yn(isset(Box::$ready)), yn(empty(Box::$ready)), "\n";
line("instance", new ReflectionProperty(Box::class, "id"));
line("static", new ReflectionProperty(Box::class, "ready"));

$box->id = 42;
$box->name = null;
$box->ratio = 2;
$box->peer = new Peer();
Box::$ready = true;
Box::$label = null;

echo "assigned|", yn(isset($box->id)), yn(empty($box->id)), yn(isset(Box::$ready)), yn(empty(Box::$ready)), "\n";
echo "values|", $box->id, "|", ($box->name === null ? "null" : $box->name), "|", $box->ratio, "|", get_class($box->peer), "|", yn(Box::$ready), "|", (Box::$label === null ? "null" : Box::$label);
