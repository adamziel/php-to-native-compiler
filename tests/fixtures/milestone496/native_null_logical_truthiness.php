<?php
echo (null && true) ? 1 : 0, "\n";
echo (null || true) ? 1 : 0, "\n";
echo (null xor true) ? 1 : 0, "\n";
echo (false || null) ? 1 : 0, "\n";
echo (true && null) ? 1 : 0;
