<?php
class ChildPlugin extends LoadedBase implements LoadedContract {
    public function label() {
        return $this->name;
    }

    public function boot() {
        return "boot";
    }
}
