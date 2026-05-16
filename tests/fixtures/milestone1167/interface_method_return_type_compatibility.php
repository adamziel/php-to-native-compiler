<?php
interface Provider {
    public function label(): string;
}

interface UntypedProvider {
    public function id();
}

class ExactProvider implements Provider {
    public function label(): string {
        return "label";
    }
}

class AddingProvider implements UntypedProvider {
    public function id(): string {
        return "id";
    }
}

echo "registered";
