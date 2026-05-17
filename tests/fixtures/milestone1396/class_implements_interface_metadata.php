<?php
interface RootHook {}
interface ChildHook extends RootHook {}
interface ParentHook {}

class ParentService implements ParentHook {}
class Service extends ParentService implements ChildHook {}

$service = new Service();
print_r(class_implements($service));
print_r(class_implements("Service", false));

$call = "class_implements";
$dynamic = $call("Service");
echo count($dynamic), "\n";
echo isset($dynamic["RootHook"]) ? "root" : "missing";
