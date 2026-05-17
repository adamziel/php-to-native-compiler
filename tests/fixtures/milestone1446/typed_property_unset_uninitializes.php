<?php
class Box {
    public int $id;
    public ?string $label;
    public $legacy = "warm";

    public function __unset($property) {
        echo "magic=", $property;
    }
}

function yn($value) {
    return $value ? "1" : "0";
}

$box = new Box();
$box->id = 42;
$box->label = "plugin";
echo "before|", yn(isset($box->id)), yn(empty($box->id)), "|", count(get_object_vars($box)), "\n";

unset($box->id);
unset($box->label);
echo "after|", yn(isset($box->id)), yn(empty($box->id)), yn(isset($box->label)), yn(empty($box->label)), "|", count(get_object_vars($box)), "\n";

$box->id = 7;
$box->label = null;
echo "reassign|", $box->id, "|", ($box->label === null ? "null" : $box->label), "|", count(get_object_vars($box)), "\n";

unset($box->missing);
