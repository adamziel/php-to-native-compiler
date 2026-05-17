<?php
trait ObjectLoadedTrait {
    public function hook() {
        return "hook:" . get_class($this);
    }
}
