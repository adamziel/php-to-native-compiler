<?php
$missing_extensions = array(
    sprintf(
        'WordPress %1$s requires the <code>%2$s</code> PHP extension.',
        "6.9.4",
        "json"
    ),
    sprintf(
        'WordPress %1$s requires the <code>%2$s</code> PHP extension.',
        "6.9.4",
        "mysqli"
    ),
);
echo implode('<br>', $missing_extensions);

