<?php
class Mailer {
    public static $validator;
}

class ChildMailer extends Mailer {
}

$mailer = new Mailer();
$mailer::$validator = 'object';
echo Mailer::$validator;
echo '|';

$class = 'ChildMailer';
$class::$validator = 'class-string';
echo ChildMailer::$validator;
echo '|';

function install_validator($phpmailer) {
    $phpmailer::$validator = static function ($email) {
        return true;
    };
}

echo 'closure-parsed';
