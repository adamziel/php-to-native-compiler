<?php
class WP_RefCow_Normal_Magic_Read_Box {
    private $secret = "private";
    protected $settings = "protected";

    public function __get($property) {
        echo "get:$property\n";
        if ($property === "secret") {
            return "magic:" . $this->secret;
        }
        if ($property === "settings") {
            return "magic:" . $this->settings;
        }
        return "missing:" . $property;
    }
}

$box = new WP_RefCow_Normal_Magic_Read_Box();
echo $box->secret, "\n";

$property = "settings";
echo $box->{$property}, "\n";

$property = "dynamic";
echo $box->{$property}, "\n";

echo $box->missing;
