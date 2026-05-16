<?php
interface Provider {
    public function label(): string;
}

class Service implements Provider {
    public function label() {}
}
