<?php
class Hook {}
class ActionHook extends Hook {}
interface HookContract {}
class ContractHook implements HookContract {}

class Registry {
    public HookLateAlias $instance;
    public static HookLateAlias $shared;
    public HookContractLateAlias $contract;
    public static HookContractLateAlias $staticContract;
}

$hook = new Hook();
$action = new ActionHook();
$contract = new ContractHook();

class_alias("Hook", "HookLateAlias");
class_alias("HookContract", "HookContractLateAlias");

$registry = new Registry();
$registry->instance = $hook;
Registry::$shared = $action;
$registry->contract = $contract;
Registry::$staticContract = $contract;

echo "instance|", get_class($registry->instance), "\n";
echo "static|", get_class(Registry::$shared), "\n";
echo "interface|", get_class($registry->contract), "\n";
echo "static-interface|", get_class(Registry::$staticContract);
