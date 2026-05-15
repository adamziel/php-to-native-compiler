<?php
function join_names($first, $second = "Grace") {
    return $first . "+" . $second;
}
echo call_user_func_array("join_names", array("Ada", "Linus")), "\n";
echo call_user_func_array("str_replace", array(" ", "_", "hello world")), "\n";
$call = "call_user_func_array";
echo $call("strlen", array("four")), "\n";
class Formatter {
    public $prefix = ">";

    public function wrap($value) {
        return $this->prefix . $value;
    }

    public static function join($left, $right) {
        return $left . ":" . $right;
    }
}
$formatter = new Formatter();
echo call_user_func_array(array($formatter, "wrap"), array("item")), "\n";
echo call_user_func_array(array("Formatter", "join"), array("a", "b"));
