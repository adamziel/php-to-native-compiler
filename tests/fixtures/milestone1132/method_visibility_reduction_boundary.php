<?php
class Base {
    public function label() {
        return "base";
    }
}

class Child extends Base {
    protected function label() {
        return "child";
    }
}
