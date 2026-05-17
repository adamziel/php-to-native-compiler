<?php
$prefix = "wp";
$callback = function ($hook, $priority = 10) use ($prefix) {
    return $prefix . ":" . $hook . ":" . $priority;
};
$prefix = "changed";

echo $callback("init"), "\n";
echo call_user_func($callback, "plugins_loaded", 5), "\n";
echo call_user_func_array($callback, array("save_post", 20)), "\n";
echo (new ReflectionFunction($callback))->invoke("admin_init");
