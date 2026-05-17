<?php
class Hook {}
class ActionHook extends Hook {}
interface HookContract {}
class ContractHook implements HookContract {}

class_alias("Hook", "HookAlias");
class_alias("HookContract", "HookContractAlias");

class Registry {
    public HookAlias $instance;
    public static HookAlias $shared;
    public HookContractAlias $contract;
    public static HookContractAlias $staticContract;
}

$registry = new Registry();
$registry->instance = new Hook();
Registry::$shared = new ActionHook();
$registry->contract = new ContractHook();
Registry::$staticContract = new ContractHook();

echo "instance|", get_class($registry->instance), "\n";
echo "static|", get_class(Registry::$shared), "\n";
echo "interface|", get_class($registry->contract), "\n";
echo "static-interface|", get_class(Registry::$staticContract);
