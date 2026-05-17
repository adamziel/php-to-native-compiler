<?php
class InvokePlugin implements InvokeLoadedContract {
    use InvokeLoadedTrait;

    public function boot() {
        return $this->hook();
    }
}
