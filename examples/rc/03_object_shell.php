<?php

class Counter {
    public static $seen = 0;
    public $name = "unset";
    public $count = 0;

    public static function mark($label) {
        self::$seen = self::$seen + 1;
        return $label . ":" . self::$seen;
    }

    public function add($amount) {
        $this->count = $this->count + $amount;
        return $this->name . "=" . $this->count;
    }
}

$counter = new Counter;
$counter->name = "rc";

echo Counter::mark("boot"), "\n";
echo $counter->add(2), "\n";

$counter->count ??= 10;
echo $counter->add(3), "\n";

var_dump(class_exists("Counter"), method_exists($counter, "add"));
