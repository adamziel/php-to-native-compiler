<?php
echo str_contains('128m', 'm') ? 'm' : '-';
echo '|';
echo str_contains('128m', 'g') ? 'g' : '-';
echo '|';
echo str_contains('128m', '') ? 'empty' : '-';
