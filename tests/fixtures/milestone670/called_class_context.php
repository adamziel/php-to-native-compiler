<?php
class Base {
    public function instanceName() {
        return get_called_class() . ":" . static::class;
    }

    public static function named() {
        return get_called_class() . ":" . static::class;
    }

    public static function forwardSelf() {
        return self::named();
    }
}

class Child extends Base {
    public static function forwardParent() {
        return parent::named();
    }
}

$child = new Child();
echo $child->instanceName(), "\n";
echo Base::named(), "\n";
echo Child::named(), "\n";
echo Child::forwardSelf(), "\n";
echo Child::forwardParent();
