<?php
trait HookTools {
    public static function label($hook) {
        return static::class . ":" . __METHOD__ . ":" . $hook;
    }

    public static function relay($hook) {
        return self::label($hook) . "|" . static::label($hook);
    }
}

class Plugin {
    use HookTools;
}

$method = new ReflectionMethod(HookTools::class, "relay");
echo @$method->invoke(null, "init"), "\n";
echo @$method->invokeArgs(new Plugin(), array("save_post"));
