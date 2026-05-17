<?php
class ObjectPlugin implements ObjectLoadedContract {
    use ObjectLoadedTrait;

    public function boot() {
        return $this->hook();
    }
}
