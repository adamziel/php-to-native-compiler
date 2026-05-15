<?php
echo preg_replace('/\\\\+0+/', '', 'a\\0b\\\\00c');
echo '|';
echo preg_replace('/\\\\+0+/', '', 'keep\\\\slash');
echo '|';
echo preg_replace('/\\\\+0+/', '', '\\000x');
