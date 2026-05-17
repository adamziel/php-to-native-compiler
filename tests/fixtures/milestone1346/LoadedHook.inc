<?php
trait LoadedHook {
    public function hook() {
        return "hook:" . get_class($this);
    }
}
