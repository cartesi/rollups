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

contract!(diamond_init);
contract!(diamond_cut_facet);
contract!(diamond_loupe_facet);
contract!(erc20_contract);
contract!(erc20_portal_facet);
contract!(erc721_portal_facet);
contract!(erc1155_portal_facet);
contract!(ether_portal_facet);
contract!(fee_manager_facet);
contract!(input_facet);
contract!(output_facet);
contract!(rollups_facet);
contract!(validator_manager_facet);
contract!(bank_contract);
