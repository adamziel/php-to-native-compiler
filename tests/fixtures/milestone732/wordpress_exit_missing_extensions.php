<?php
$missing_extensions = array(
    'WordPress 6.9.4 requires the <code>json</code> PHP extension.',
    'WordPress 6.9.4 requires the <code>mysqli</code> PHP extension.',
);
echo implode('<br>', $missing_extensions);
exit(1);
echo "after";

