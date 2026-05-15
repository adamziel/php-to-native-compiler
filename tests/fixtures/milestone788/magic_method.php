<?php
echo "top:", __METHOD__, "\n";

function free_function($default = __METHOD__) {
    echo "function-default:", $default, "\n";
    echo "function-body:", __METHOD__, "\n";
}

class ParentBox {
    public function inherited() {
        return __METHOD__;
    }
}

class Box extends ParentBox {
    public function label($default = __METHOD__) {
        echo "method-default:", $default, "\n";
        echo "method-body:", __METHOD__, "\n";
    }

    public static function staticLabel() {
        echo "static-body:", __METHOD__, "\n";
    }
}

free_function();
$box = new Box();
$box->label();
echo "inherited:", $box->inherited(), "\n";
Box::staticLabel();
