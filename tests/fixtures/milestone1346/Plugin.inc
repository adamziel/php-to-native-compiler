<?php
class Plugin {
    use LoadedHook;

    public function boot() {
        return $this->hook();
    }
}
