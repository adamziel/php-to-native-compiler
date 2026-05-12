<?php
class Box {
    public function label() {
        return __METHOD__;
    }
}
$box = new Box();
echo "unreachable";
