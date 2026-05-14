<?php
const GLOBAL_SUFFIX = "global";
class Box {}
class Root {
    public const ROOT = "root";
    protected const PROTECTED_NAME = "protected";
    private const SECRET = "secret";
    public const LABEL = Box::class;
    public const SUM = 7 + 5;
    public const FROM_GLOBAL = GLOBAL_SUFFIX;

    public function rootNames() {
        return self::ROOT . ":" . self::SECRET;
    }
}
class Base extends Root {
    public const BASE = "base";

    public function baseNames() {
        return self::BASE . ":" . parent::ROOT . ":" . parent::PROTECTED_NAME;
    }
}
class Child extends Base {
    public function childNames() {
        return self::BASE . ":" . parent::BASE . ":" . Root::ROOT . ":" . Root::LABEL . ":" . Root::SUM . ":" . Root::FROM_GLOBAL;
    }
}

$child = new Child();
echo Root::ROOT, "\n";
echo $child->rootNames(), "\n";
echo $child->baseNames(), "\n";
echo $child->childNames(), "\n";
echo "done";
