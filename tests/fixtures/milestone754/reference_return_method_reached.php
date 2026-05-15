<?php
class Factory {
    public function &make_entry($value) {
        return $value;
    }
}

$factory = new Factory();
echo $factory->make_entry(1);
