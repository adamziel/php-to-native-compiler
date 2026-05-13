<?php
echo __LINE__, "\n";
echo __FILE__, "\n";
echo __DIR__, "\n";
echo __FUNCTION__, "\n";
function current_magic($default = __FUNCTION__) {
    echo $default, "\n";
    echo __FUNCTION__;
}
current_magic();
