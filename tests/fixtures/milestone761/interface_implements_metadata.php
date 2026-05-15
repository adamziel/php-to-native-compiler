<?php
interface Logger {}
interface Hookable {}

class Service implements Logger, Hookable {}
class ChildService extends Service {}

$service = new Service();
$child = new ChildService();

echo is_a($service, "Logger") ? "service:logger\n" : "service:no\n";
echo is_subclass_of($service, "Hookable") ? "service:hookable\n" : "service:no\n";
echo is_a($child, "Logger") ? "child:logger" : "child:no";
