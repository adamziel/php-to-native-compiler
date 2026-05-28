<?php
namespace FirstNS;
use Vendor\Tool as Alias;

function label() {
    return __FUNCTION__;
}

echo Alias::class, "\n";
echo label(), "\n";

namespace SecondNS;

function label() {
    return __FUNCTION__;
}

echo Alias::class, "\n";
echo label(), "\n";

namespace FirstNS;

echo Alias::class, "\n";
echo label();
