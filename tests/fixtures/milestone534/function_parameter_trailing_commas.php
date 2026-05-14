<?php
function identity($value,) {
    return $value;
}

function label($name, $suffix = "!",) {
    return $name . $suffix;
}

class Box {
    public function method($value,) {
        return $value;
    }
}

echo identity("Ada"), "\n";
echo label("Grace"), "\n";
echo label("Lin", ".");
