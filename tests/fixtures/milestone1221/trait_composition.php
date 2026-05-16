<?php
trait Labels {
    public function label($value = "default") {
        return "label:" . $value;
    }

    function receiver() {
        return get_class($this);
    }
}

class Widget {
    use Labels;
}

$widget = new Widget();
echo $widget->label("ok"), "|";
echo $widget->label(), "|";
echo $widget->receiver(), "|";
echo trait_exists("Labels") ? "trait" : "missing";
