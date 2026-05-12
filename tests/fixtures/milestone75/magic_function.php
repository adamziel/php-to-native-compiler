<?php
echo "top:", __FUNCTION__, "\n";

function current_name($default = __FUNCTION__) {
    echo "default:", $default, "\n";
    echo "body:", __FUNCTION__, "\n";
}

function caller() {
    current_name();
    echo "caller:", __FUNCTION__, "\n";
}

current_name("manual");
caller();
