<?php
class Compat {
    public static function add(
        #[\SensitiveParameter]
        $val,
        #[\SensitiveParameter]
        $addv
    ) {
        return $val . $addv;
    }
}

echo Compat::add("A", "B");
