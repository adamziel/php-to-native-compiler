<?php
$iso_8859_1_text = 123;
$iso_8859_1_text = (string) $iso_8859_1_text;
echo $iso_8859_1_text, "\n";
echo "[", (string) null, "]\n";
echo "[", (string) false, "]\n";
echo (string) true, "|", (string) 3.5, "|", (string) "utf8";
