<?php
class Box {
    public function label() {
        return __TRAIT__;
    }
}
$box = new Box();
echo "unreachable";
