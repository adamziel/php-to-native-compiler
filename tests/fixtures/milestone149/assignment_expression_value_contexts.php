<?php
function capture($left, $right) {
    echo "capture:", $left, ":", $right, "\n";
    return $left . "|" . $right;
}

$arg = "start";
echo capture(($arg = "call"), ($arg .= "-arg")), ":", $arg, "\n";

$array = [
    ($key = "name") => ($value = "Ada"),
    ($next = 2) => ($value = $value . "-Lovelace"),
];
echo "array:", $key, ":", $next, ":", $array["name"], ":", $array[2], ":", $value, "\n";

if (($condition = strlen(($text = "php"))) === 3) {
    echo "if:", $condition, ":", $text, "\n";
}

echo "coalesce:", strlen(($maybe ??= "seed")), ":", $maybe, "\n";

$loop = 0;
while (($loop += 1) < 3) {
    echo "while:", $loop, "\n";
}
echo "after-while:", $loop, "\n";

for ($i = 0; ($gate = $i < 2); $i = $i + 1) {
    echo "for:", $i, ":", $gate, "\n";
}

$items = [];
echo "builtin:",
    array_key_exists(($lookup = "slot"), ($items = ["slot" => "yes"])),
    ":", $lookup,
    ":", $items["slot"],
    ":", count(($copy = [1, 2, 3])),
    ":", count($copy);
