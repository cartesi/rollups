local name = "SpliceMachineEnum"

local sol_version = "^0.8.13"

local imports = {
    "./src/SpliceMachine.sol",
}

local variants = {
    { name = "WaitingSpliceClaim", typ = "SpliceMachine.WaitingSpliceClaim" },
    { name = "WaitingAgreement", typ = "SpliceMachine.WaitingAgreement" },
}

return {
    name = name,
    sol_version = sol_version,
    imports = imports,
    variants = variants,
}
