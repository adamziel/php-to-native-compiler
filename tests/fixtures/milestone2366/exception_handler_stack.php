<?php
function first_handler($e) {
    echo "first:", $e->getMessage();
}

class HandlerBox {
    public static function second($e) {
        echo "second:", $e::class, ":", $e->getMessage(), "\n";
    }
}

var_dump(get_exception_handler());
var_dump(set_exception_handler("first_handler"));

$previous = set_exception_handler(["HandlerBox", "second"]);
echo is_string($previous) ? "prev:$previous\n" : "bad\n";

restore_exception_handler();
throw new Exception("boom");
