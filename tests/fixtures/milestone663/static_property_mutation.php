<?php
class Counter {
    public static $count;
    public static $label;
    public static $maybe;
}
class Base {
    protected static $shared;
}
class Child extends Base {
    public static $own;

    public function run() {
        parent::$shared = 5;
        parent::$shared += 2;
        echo parent::$shared++, "\n";
        echo parent::$shared, "\n";
        echo ++parent::$shared, "\n";
        self::$own ??= "child";
        self::$own ??= "again";
        return parent::$shared . ":" . self::$own;
    }
}
class LoopCounter {
    public static $i;
}

Counter::$count = 1;
$updated = (Counter::$count += 4);
echo $updated, "\n";
echo Counter::$count++, "\n";
echo ++Counter::$count, "\n";
Counter::$label = "a";
Counter::$label .= "b";
echo Counter::$label, "\n";
$first = (Counter::$maybe ??= "first");
$second = (Counter::$maybe ??= "second");
echo $first, "\n";
echo $second, "\n";
$child = new Child();
echo $child->run(), "\n";
for (LoopCounter::$i = 0; LoopCounter::$i < 2; LoopCounter::$i++) {
    echo "loop", LoopCounter::$i, "\n";
}
