<?php
$missing = null;
echo true ? "yes" : fail(), "\n";
echo false ?: "fallback", "\n";
echo $missing ?? "coalesced", "\n";
$value = "present";
echo $value ?? fail();
