<?php
trait HookTools {
    public static function tag($hook, $priority = 10) {
        return "trait:" . $hook . ":" . $priority;
    }
}

class Plugin {
    use HookTools;
}

$method = new ReflectionMethod(HookTools::class, "tag");
echo $method->invoke(null, "init"), "\n";
echo $method->invokeArgs(new Plugin(), array("save_post", 20));
