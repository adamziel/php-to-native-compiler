<?php
echo 'top:', __CLASS__, "\n";

function label()
{
    return __CLASS__;
}

class Box
{
    public function label()
    {
        return __CLASS__;
    }
}

$box = new Box();
echo 'function:', label(), "\n";
echo 'method:', $box->label();
