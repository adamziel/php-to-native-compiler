<?php
interface HookTarget {}
interface ChildHookTarget extends HookTarget {}
class BaseTarget implements HookTarget {}
class ChildTarget extends BaseTarget implements ChildHookTarget {}

interface ParentResolver {
    public function bind(ChildHookTarget $target): HookTarget;
}

interface ChildResolver extends ParentResolver {
    public function bind(HookTarget $target): ChildHookTarget;
}

interface Resolver {
    public function resolve(ChildTarget $target): BaseTarget;
}

class BaseResolver {
    public function resolve(ChildTarget $target): BaseTarget {
        return $target;
    }
}

class PluginResolver extends BaseResolver implements Resolver {
    public function resolve(BaseTarget $target): ChildTarget {
        return new ChildTarget();
    }
}

class InterfaceResolver implements Resolver {
    public function resolve(HookTarget $target): ChildTarget {
        return new ChildTarget();
    }
}

class InterfaceParentResolver {
    public function bind(ChildHookTarget $target): HookTarget {
        return $target;
    }
}

class InterfaceChildResolver extends InterfaceParentResolver {
    public function bind(HookTarget $target): ChildHookTarget {
        return new ChildTarget();
    }
}

echo method_exists(new PluginResolver(), "resolve") ? "inherited:registered\n" : "inherited:missing\n";
echo method_exists(new InterfaceResolver(), "resolve") ? "interface:registered\n" : "interface:missing\n";
echo interface_exists("ChildResolver") ? "child-interface:registered\n" : "child-interface:missing\n";
echo method_exists(new InterfaceChildResolver(), "bind") ? "interface-parent:registered" : "interface-parent:missing";
