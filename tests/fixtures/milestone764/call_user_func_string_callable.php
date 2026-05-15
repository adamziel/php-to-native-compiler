<?php
function greet($name) {
    return "hi " . $name;
}
echo call_user_func("greet", "Ada"), "\n";
echo call_user_func("str_replace", " ", "_", "hello world"), "\n";
$call = "call_user_func";
echo $call("strlen", "four");
