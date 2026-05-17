<?php
interface HookContract {}
interface ChildHookContract extends HookContract {}

class ActionHook implements ChildHookContract {}
class FilterHook extends ActionHook {}

class Registry {
    public HookContract $instance;
    public static ChildHookContract $shared;
}

$registry = new Registry();
$registry->instance = new ActionHook();
Registry::$shared = new FilterHook();

echo "instance|", get_class($registry->instance), "\n";
echo "static|", get_class(Registry::$shared);
