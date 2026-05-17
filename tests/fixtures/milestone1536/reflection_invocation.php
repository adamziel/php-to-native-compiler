<?php
function render_hook($tag, $suffix = "done") {
    return $tag . ":" . $suffix;
}

class HookRunner {
    public $log = array();

    public function append($hook, $priority = 10) {
        $this->log[] = $hook . ":" . $priority;
        return count($this->log);
    }
}

$function = new ReflectionFunction("render_hook");
echo $function->invoke("init"), "\n";
echo $function->invokeArgs(array("save_post", "later")), "\n";

$runner = new HookRunner();
$method = new ReflectionMethod(HookRunner::class, "append");
echo $method->invoke($runner, "init"), "|", implode(",", $runner->log), "\n";
echo $method->invokeArgs($runner, array("save_post", 20)), "|", implode(",", $runner->log);
