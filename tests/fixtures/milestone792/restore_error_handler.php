<?php
function quiet_error_handler()
{
}
set_error_handler('quiet_error_handler', E_WARNING);
echo restore_error_handler() ? 'restored' : 'failed';
$previous = set_error_handler('quiet_error_handler');
echo '|';
echo $previous === null ? 'null' : 'other';
