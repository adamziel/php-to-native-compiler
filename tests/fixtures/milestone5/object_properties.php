<?php
class Profile {
    public $name;
    public $visits;
    protected $secret;
    private $token;
    private static $cache;
}

$profile = new profile();
echo "initial:", $profile->name, "\n";
$profile->name = "Ada";
$profile->visits = 3;
echo $profile->name, "\n";
echo $profile->visits + 2, "\n";
print_r($profile);
