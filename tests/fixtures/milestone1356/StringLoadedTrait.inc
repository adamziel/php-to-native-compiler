<?php
trait StringLoadedTrait {
    public function hook() {
        return "hook:" . get_class($this);
    }
}
