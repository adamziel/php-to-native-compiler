<?php
$items = [
    "first",
    2 => "two",
    "2" => "two updated",
    "02" => "zero two",
    "name" => "Ada",
    1 + 2 => "three",
];
echo count($items), "\n";
print_r($items);
if ($items) {
    echo "truthy\n";
}
if ([]) {
    echo "bad\n";
} else {
    echo "empty\n";
}
