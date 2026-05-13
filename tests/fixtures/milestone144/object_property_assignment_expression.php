<?php
class Box {
    public $name;
    public $count;
    public $result;
}

$box = new Box();
echo ($box->name = "Ada"), ":", $box->name, "\n";
echo ($box->count = 41 + 1), ":", $box->count, "\n";

function rhs_value() {
    echo "rhs\n";
    return "value";
}

echo ($box->result = rhs_value()), ":", $box->result;
