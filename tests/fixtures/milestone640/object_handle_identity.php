<?php
class Box {
    public $value;
}

function set_value($object, $value) {
    $object->value = $value;
    return $object;
}

$box = new Box();
$alias = $box;
$alias->value = "alias";
echo $box->value, "\n";

$items = [$box];
$fromArray = $items[0];
$fromArray->value = "array";
echo $box->value, "\n";

set_value($box, "function");
echo $box->value, "\n";

$returned = set_value($box, "return");
echo $box->value, "\n";

foreach ([$box] as $item) {
    $item->value = "foreach";
}
echo $box->value, "\n";

var_dump($box === $alias);
var_dump($box === $returned);
var_dump($box === new Box());
var_dump(spl_object_id($box) === spl_object_id($alias));
var_dump(spl_object_hash($box) === spl_object_hash($alias));
echo "done";
