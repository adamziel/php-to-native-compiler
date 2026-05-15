<?php
echo (float) null, "|", (float) false, "|", (float) true, "\n";
echo (double) 42, "|", (float) -3.8, "|", (float) " 15 ", "|", (float) "2.9", "\n";
echo (float) "", "|", (float) "not numeric", "|", (float) "1e3", "\n";
echo is_float((float) "1") ? "float" : "other", "|";
echo ((double) "2.25") === 2.25 ? "double" : "other";
