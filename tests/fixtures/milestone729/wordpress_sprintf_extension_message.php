<?php
$wp_version = "6.9.4";
$extension = "json";
echo sprintf(
    'WordPress %1$s requires the <code>%2$s</code> PHP extension.',
    $wp_version,
    $extension
);
