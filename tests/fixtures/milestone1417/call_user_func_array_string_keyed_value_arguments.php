<?php
function wp_refcow_named_value($value, $suffix = "default", $prefix = "pre") {
    return $prefix . ":" . $value . ":" . $suffix;
}

class WP_RefCow_Named_Value_Callback {
    public function wrap($value, $suffix = "method", $prefix = "box") {
        return $prefix . ":" . $value . ":" . $suffix;
    }

    public static function join($value, $suffix = "static", $prefix = "class") {
        return $prefix . ":" . $value . ":" . $suffix;
    }
}

echo call_user_func_array("wp_refcow_named_value", array("suffix" => "literal", "value" => "cache")), "\n";

$stored = array("prefix" => "stored", "value" => "option");
echo call_user_func_array("wp_refcow_named_value", $stored), "\n";

$callback = new WP_RefCow_Named_Value_Callback();
echo call_user_func_array(array($callback, "wrap"), array("suffix" => "object", "value" => "payload")), "\n";
echo call_user_func_array(array("WP_RefCow_Named_Value_Callback", "join"), array("prefix" => "static", "value" => "payload"));
