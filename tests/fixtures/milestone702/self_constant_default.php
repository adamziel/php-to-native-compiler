<?php
class Defaults {
    const SIZE = 32;
    private const SECRET = "secret";

    public static function stat($length = self::SIZE) {
        echo $length, "\n";
    }

    public function inst($label = self::SECRET, $suffix = ":" . self::SIZE) {
        echo $label, $suffix, "\n";
    }
}

class BaseDefaults {
    const SIZE = 16;

    public static function inherited($length = self::SIZE) {
        echo $length, "\n";
    }
}

class ChildDefaults extends BaseDefaults {
    const SIZE = 64;
}

Defaults::stat();
Defaults::stat(64);
$defaults = new Defaults();
$defaults->inst();
$defaults->inst("manual", ":8");
ChildDefaults::inherited();
ChildDefaults::inherited(128);
