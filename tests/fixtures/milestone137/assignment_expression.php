<?php
echo ($value = 10), ":", $value, "\n";
echo ($value = $value + 5), ":", $value, "\n";
echo (($text = "php") . "-native"), ":", $text, "\n";
echo ($array = ["name" => "Ada"])["name"], ":", $array["name"], "\n";

function next_value() {
    echo "rhs\n";
    return 42;
}
echo ($value = next_value()), ":", $value;
