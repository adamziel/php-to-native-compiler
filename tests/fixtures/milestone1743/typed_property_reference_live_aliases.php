<?php
class Hook {}
class ActionHook extends Hook {}
interface HookContract {}
class ContractHook implements HookContract {}

class Registry {
    public HookLateAlias $instance;
    public HookContractLateAlias $contract;
}

$hook = new Hook();
$action = new ActionHook();
$contract = new ContractHook();

class_alias("Hook", "HookLateAlias");
class_alias("HookContract", "HookContractLateAlias");

$registry = new Registry();
$registry->instance = $hook;
$registry->contract = $contract;

$instance =& $registry->instance;
$instance = $action;
echo get_class($registry->instance), "|", get_class($instance), "\n";

$target = array();
$target["copy"] =& $instance;
$target["copy"] = $hook;
echo get_class($registry->instance), "|", get_class($target["copy"]), "\n";

$contractAlias =& $registry->contract;
$contractAlias = $contract;
echo get_class($registry->contract), "|", get_class($contractAlias);
