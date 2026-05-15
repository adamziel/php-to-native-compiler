<?php
echo is_dir(__DIR__) ? 'dir' : 'missing';
echo '|';
echo is_dir(__FILE__) ? 'dir' : 'file';
echo '|';
echo is_dir(__DIR__ . '/missing-dir') ? 'dir' : 'missing';
