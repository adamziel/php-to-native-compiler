<?php
$value = "2";
switch ($value) {
    case 1:
        echo "one";
        break;
    case 2:
        echo "two";
    default:
        echo "-default";
    case "tail":
        echo "-tail";
        break;
}
echo "\n";

$word = "none";
switch ($word) {
    default:
        echo "fallback";
    case "none":
        echo "matched";
        break;
}
echo "\n";

$missing = "missing";
switch ($missing) {
    default:
        echo "fallback";
    case "after":
        echo "-after";
        break;
}
echo "\n";

$i = 0;
while ($i < 3) {
    switch ($i) {
        case 0:
            echo "zero";
            break;
        case 1:
            echo "one";
            break;
        default:
            echo "many";
    }
    echo ":";
    $i = $i + 1;
}
echo "\n";
