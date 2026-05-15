<?php
class Account {
    public $id;
}

class Profile extends Account {
    public $name;
}

$profile = new Profile();
$id = "id";
$name = "name";
$profile->$id = 7;
$profile->$name = "Ada";
echo $profile->id, "|", $profile->$id, "|", $profile->$name, "\n";

$data = new stdClass();
$key = "answer";
$data->$key = 42;
echo $data->answer, "|", $data->$key, "\n";

$intKey = 7;
$data->$intKey = "seven";
echo $data->$intKey;
