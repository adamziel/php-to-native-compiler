<?php
function quiet_error_handler() {
}
$previous = set_error_handler('quiet_error_handler');
echo $previous === null ? 'null' : 'other';

class Handler {
    public function handle() {
    }
}
$handler = new Handler();
$previous = set_error_handler([$handler, 'handle'], E_WARNING);
echo '|';
echo is_string($previous) ? $previous : 'other';

$previous = set_error_handler(function () {
    echo 'not-now';
});
echo '|';
echo is_array($previous) ? 'array' : 'other';
echo '|body';
