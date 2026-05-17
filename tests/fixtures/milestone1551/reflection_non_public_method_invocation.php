<?php
class BaseHook {
    private function privateTag($hook) {
        $this->log[] = "private:" . $hook . ":" . static::class;
        return count($this->log);
    }

    protected function protectedTag($hook, $priority = 10) {
        $this->log[] = "protected:" . $hook . ":" . $priority . ":" . static::class;
        return count($this->log);
    }

    protected static function staticTag($hook) {
        return "static:" . $hook . ":" . static::class;
    }
}

class ChildHook extends BaseHook {
    public $log = array();
}

$child = new ChildHook();

$private = new ReflectionMethod(BaseHook::class, "privateTag");
echo $private->invoke($child, "init"), "|", implode(",", $child->log), "\n";

$protected = new ReflectionMethod(BaseHook::class, "protectedTag");
echo $protected->invokeArgs($child, array("save_post", 20)), "|", implode(",", $child->log), "\n";

$static = new ReflectionMethod(ChildHook::class, "staticTag");
echo $static->invoke(null, "shutdown");
