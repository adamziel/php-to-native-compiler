<?php
trait Base {
    public function render() {}
}

trait Reusable {
    use Base { Base::render as alias; }
}
