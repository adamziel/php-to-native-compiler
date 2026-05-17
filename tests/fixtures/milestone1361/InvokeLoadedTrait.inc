<?php
trait InvokeLoadedTrait {
    public function hook() {
        return "hook:" . get_class($this);
    }
}
