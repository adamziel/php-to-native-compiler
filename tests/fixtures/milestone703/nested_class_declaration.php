<?php
if (false) {
    class SkippedNestedBox {}
}

echo class_exists("SkippedNestedBox"), "\n";

if (true) {
    class ExecutedNestedBox {
        public $value;

        public function label() {
            return "nested:" . $this->value;
        }
    }
}

$box = new ExecutedNestedBox();
$box->value = 42;
echo class_exists("ExecutedNestedBox"), "\n";
echo $box->label(), "\n";

function declare_function_box() {
    class FunctionNestedBox {
        public static function label() {
            return "function";
        }
    }
}

echo class_exists("FunctionNestedBox"), "\n";
declare_function_box();
echo class_exists("FunctionNestedBox"), "\n";
echo FunctionNestedBox::label(), "\n";
