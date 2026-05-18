<?php
class TypedRefBox {
    public int $id;
    public ?string $label;
    private int $hidden = 4;

    public function &hiddenRef() {
        $hidden =& $this->hidden;
        return $hidden;
    }

    public function hiddenValue() {
        return $this->hidden;
    }

    public function setHidden($value) {
        $this->hidden = $value;
    }
}

function assign_ref(&$value, $next) {
    $value = $next;
}

$box = new TypedRefBox();
$box->id = 1;
$box->label = null;

$property = "id";
$id =& $box->{$property};
$id = "2";
echo gettype($box->id), ":", $box->id, "|", gettype($id), ":", $id, "\n";

$copy = clone $box;
$copy->id = "3";
echo gettype($id), ":", $id, "|", gettype($box->id), ":", $box->id, "|", gettype($copy->id), ":", $copy->id, "\n";

assign_ref($id, "7");
echo gettype($box->id), ":", $box->id, "|", gettype($id), ":", $id, "\n";

$other =& $id;
$other = "8";
echo gettype($box->id), ":", $box->id, "|", gettype($other), ":", $other, "\n";

$label =& $box->label;
$label = 123;
echo gettype($box->label), ":", $box->label, "|", gettype($label), ":", $label, "\n";

$hidden =& $box->hiddenRef();
$hidden = "5";
echo gettype($hidden), ":", $hidden, "|", gettype($box->hiddenValue()), ":", $box->hiddenValue(), "\n";

$hiddenCopy = clone $box;
$hiddenCopy->setHidden("6");
echo gettype($hidden), ":", $hidden, "|", gettype($box->hiddenValue()), ":", $box->hiddenValue(), "|", gettype($hiddenCopy->hiddenValue()), ":", $hiddenCopy->hiddenValue();
