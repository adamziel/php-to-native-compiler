<?php
#[Example]
function demo(#[SensitiveParameter] $value) {
    return $value;
}

#[Example]
class Box {
    #[Example]
    public function label(#[SensitiveParameter] $value) {
        return $value;
    }
}

echo demo("ok"), "\n";
$box = new Box();
echo $box->label("box");
