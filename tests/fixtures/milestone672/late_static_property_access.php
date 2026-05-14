<?php
class Base {
    public static $shared = "base-default";
    public static $maybe;
    public static $count;
    protected static $secret;

    public static function seed($value) {
        static::$shared = $value;
        static::$secret = static::class . ":secret";
        echo static::$shared, ":", static::$secret, "\n";
        static::$shared .= ":x";
        echo static::$shared, "\n";
        static::$count ??= 0;
        static::$count += 2;
        echo static::$count++, ":", static::$count, "\n";
        echo isset(static::$shared) ? "shared:set\n" : "shared:unset\n";
        echo empty(static::$missing) ? "missing:empty\n" : "missing:not-empty\n";
        echo static::$missing ?? "fallback", "\n";
        static::$maybe ??= "maybe";
        echo static::$maybe, "\n";
    }
}

class Child extends Base {
    public static $shared = "child-default";
    public static $maybe;
    public static $count;

    public static function callParentSeed() {
        parent::seed("parent-child");
    }
}

Base::seed("base");
Child::seed("child");
Child::callParentSeed();
echo Base::$shared, "\n";
echo Child::$shared, "\n";
