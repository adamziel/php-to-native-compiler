<?php
namespace App\Core;

class Base {}
class Service extends Base {}

$service = new Service();
echo Service::class, "\n";
echo get_class($service), "\n";
echo get_parent_class($service);
