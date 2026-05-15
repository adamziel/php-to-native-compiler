<?php
class Account {
    public $id;
}

$name = 'id';
$account = new Account();
$account->{$name} = 7;
echo $account->{'i' . 'd'};
echo '|';

$data = new stdClass();
$slot = 'answer';
$data->{$slot} = 42;
echo $data->{$slot};
