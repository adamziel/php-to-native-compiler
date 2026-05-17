<?php
trait HookTools {
    public static function context($hook) {
        return __CLASS__ . "|" . __METHOD__ . "|" . self::class . "|" . static::class . "|" . get_called_class() . "|" . $hook;
    }
}

class Plugin {
    use HookTools;
}

$method = new ReflectionMethod(HookTools::class, "context");
echo $method->invoke(null, "init"), "\n";
echo $method->invokeArgs(new Plugin(), array("save_post"));
