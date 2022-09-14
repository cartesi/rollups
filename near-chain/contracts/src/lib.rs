/// Declares `$contract_name` as a module and includes everything from the `$contract_name` ABI.
macro_rules! contract {
    ($contract_name: ident) => {
        pub mod $contract_name {
            include!(concat!(
                env!("OUT_DIR"),
                "/",
                stringify!($contract_name),
                ".rs"
            ));
        }
    };
}

contract!(input_box);
contract!(cartesi_dapp);
contract!(authority);
contract!(history);
