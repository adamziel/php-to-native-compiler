<?php
echo false ?: 42, "\n";
echo false ?: 2.5, "\n";
echo false ?: "fallback", "\n";
echo "a", false ?: null, "b", "\n";
echo true ?: [];
