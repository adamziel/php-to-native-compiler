<?php
echo (int) null, "|", (int) false, "|", (int) true, "\n";
echo (integer) 42, "|", (int) -3.8, "|", (int) " 15 ", "|", (int) "2.9", "\n";
echo (int) "", "|", (int) "not numeric", "|", (int) "+.", "|", (int) "128m", "|", (int) "1.2e3m";
