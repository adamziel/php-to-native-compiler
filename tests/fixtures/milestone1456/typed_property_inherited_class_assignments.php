<?php
class Hook {}
class ActionHook extends Hook {}
class FilterHook extends ActionHook {}

class Registry {
    public Hook $instance;
    public static Hook $shared;
}

$registry = new Registry();
$registry->instance = new ActionHook();
Registry::$shared = new FilterHook();

echo "instance|", get_class($registry->instance), "\n";
echo "static|", get_class(Registry::$shared);
