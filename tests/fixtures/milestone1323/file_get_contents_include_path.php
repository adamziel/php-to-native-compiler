<?php
set_include_path(__DIR__ . "/include_path_lib");
echo trim(file_get_contents("wp_loader.inc", true));
