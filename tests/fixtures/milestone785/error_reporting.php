<?php
echo error_reporting();
echo '|';
echo error_reporting(0);
echo '|';
echo error_reporting();
echo '|';
echo error_reporting(E_CORE_ERROR | E_CORE_WARNING | E_COMPILE_ERROR | E_ERROR | E_WARNING | E_PARSE | E_USER_ERROR | E_USER_WARNING | E_RECOVERABLE_ERROR);
echo '|';
echo error_reporting();
