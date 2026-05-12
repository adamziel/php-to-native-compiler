<?php
function bit($value) {
    if ($value) {
        return "1";
    }
    return "0";
}

function row($label, $left, $right) {
    echo $label, ":",
        bit($left === $right),
        bit($left !== $right),
        "\n";
}

row("null|null", null, null);
row("null|false", null, false);
row("false|false", false, false);
row("false|int0", false, 0);
row("true|int1", true, 1);
row("int1|int1", 1, 1);
row("int1|float1", 1, 1.0);
row("float1|float1", 1.0, 1.0);
row("str1|int1", "1", 1);
row("str1|str1", "1", "1");
