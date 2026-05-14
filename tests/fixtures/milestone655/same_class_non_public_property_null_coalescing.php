<?php
class Box {
    private $secret;
    protected $label;

    public function seed($secret, $label) {
        $this->secret = $secret;
        $this->label = $label;
    }

    public function coalesce($other) {
        echo ($this->secret ?? "secret-fallback"), "\n";
        echo ($this->missing ?? "missing-fallback"), "\n";
        $this->secret ??= "secret-assigned";
        $this->label ??= "label-replaced";
        $other->secret ??= "peer-secret";
        $other->label ??= "peer-label";
        echo $this->secret, ":", $this->label, "\n";
        echo $other->secret, ":", $other->label;
    }
}

$first = new Box();
$second = new Box();
$first->seed(null, "kept");
$second->seed("existing", null);
$first->coalesce($second);
