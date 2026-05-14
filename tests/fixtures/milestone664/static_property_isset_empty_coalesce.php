<?php
class Flags {
    public static $nullable;
    public static $flag;
    public static $zero;
    public static $blank;
    public static $text;
}
class Base {
    protected static $secret;
    public static $shared;
}
class Child extends Base {
    public function run() {
        parent::$secret = "hidden";
        self::$shared = "shared";
        echo isset(parent::$secret) ? "secret:set\n" : "secret:unset\n";
        echo empty(parent::$secret) ? "secret:empty\n" : "secret:not-empty\n";
        echo parent::$secret ?? "secret-fallback", "\n";
        echo self::$shared ?? "shared-fallback", "\n";
    }
}

Flags::$flag = false;
Flags::$zero = 0;
Flags::$blank = "";
Flags::$text = "ok";
echo isset(Flags::$nullable) ? "nullable:set\n" : "nullable:unset\n";
echo isset(Flags::$flag) ? "flag:set\n" : "flag:unset\n";
echo isset(Flags::$zero) ? "zero:set\n" : "zero:unset\n";
echo isset(Flags::$blank) ? "blank:set\n" : "blank:unset\n";
echo isset(Flags::$text) ? "text:set\n" : "text:unset\n";
echo empty(Flags::$nullable) ? "nullable:empty\n" : "nullable:not-empty\n";
echo empty(Flags::$flag) ? "flag:empty\n" : "flag:not-empty\n";
echo empty(Flags::$zero) ? "zero:empty\n" : "zero:not-empty\n";
echo empty(Flags::$blank) ? "blank:empty\n" : "blank:not-empty\n";
echo empty(Flags::$text) ? "text:empty\n" : "text:not-empty\n";
echo Flags::$nullable ?? "fallback", "\n";
echo Flags::$text ?? "fallback", "\n";
echo isset(Flags::$missing) ? "missing:set\n" : "missing:unset\n";
echo empty(Flags::$missing) ? "missing:empty\n" : "missing:not-empty\n";
echo Flags::$missing ?? "missing-fallback", "\n";
$child = new Child();
$child->run();
