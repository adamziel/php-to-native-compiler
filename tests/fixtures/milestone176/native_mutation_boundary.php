<?php
$value = 1;
$value += 4;
echo $value, "\n";
$value ??= 99;
echo $value, "\n";
$missing ??= "created";
echo $missing, "\n";
echo ($assigned = "expr"), ":", $assigned, "\n";
echo ($value *= 2), ":", $value, "\n";
echo $value++, ":", $value, "\n";
unset($assigned, $missing);
if (isset($assigned)) {
    echo "assigned\n";
} else {
    echo "unset\n";
}
if (isset($missing)) {
    echo "missing\n";
} else {
    echo "unset-missing";
}
