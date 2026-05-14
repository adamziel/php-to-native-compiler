<?php
if (!class_exists("WP_Bootstrap_Exception")) {
    class WP_Bootstrap_Exception extends Exception {}
}

$exception = new WP_Bootstrap_Exception();
echo get_parent_class($exception), "\n";
echo is_subclass_of($exception, "Exception") ? "yes" : "no";
