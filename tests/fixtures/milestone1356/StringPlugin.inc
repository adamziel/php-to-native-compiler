<?php
class StringPlugin implements StringLoadedContract {
    use StringLoadedTrait;

    public function boot() {
        return $this->hook();
    }
}
