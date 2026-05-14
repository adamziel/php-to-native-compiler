<?php
class Counter {
    public static $count;
}
class Base {
    public static $shared;
}
class Child extends Base {
    public static $own;

    public function run() {
        parent::$shared = "base";
        self::$own = "child";
        return parent::$shared . ":" . self::$own;
    }
}

Counter::$count = 1;
echo Counter::$count, "\n";
Counter::$count = Counter::$count + 4;
echo Counter::$count, "\n";
$child = new Child();
echo $child->run(), "\n";
echo Base::$shared, "\n";
echo "done";
