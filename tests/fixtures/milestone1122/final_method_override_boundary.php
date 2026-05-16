<?php
class Base {
    final public function seal() {
        return "base";
    }
}

class Child extends Base {
    public function SEAL() {
        return "child";
    }
}
