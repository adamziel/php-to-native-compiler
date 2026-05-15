<?php
function quiet_shutdown($value) {
}

class Handler {
    public function handle() {
    }
}

$handler = new Handler();
register_shutdown_function([$handler, 'handle']);
echo 'array';
echo '|';
register_shutdown_function('quiet_shutdown', 'later');
echo 'string';
