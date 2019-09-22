use clap::load_yaml;
use codec::Decode;
use futures::stream::Stream;
use futures::Future;
use keyring::AccountKeyring;
use pwc_node_runtime::{self, uniswap::RawEvent as UniswapEvent, uniswap::Trait as UniswapTrait};
use runtime_primitives::generic::Era;
use substrate_primitives::crypto::Pair;
use substrate_subxt::{
    srml::{
        superfluid::{Superfluid, SuperfluidXt},
        system::System,
    },
    ClientBuilder,
};
use url::Url;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = clap::App::from_yaml(yaml)
        .version(env!("CARGO_PKG_VERSION"))
        .get_matches();

    execute(matches)
}

fn print_usage(matches: &clap::ArgMatches) {
    println!("{}", matches.usage());
}

fn execute(matches: clap::ArgMatches) {
    match matches.subcommand() {
        ("run", Some(matches)) => {
            let addr = matches
                .value_of("addr")
                .expect("The address of superfluid chain is required; thus it can't be None; qed");
            let addr = Url::parse(&format!("ws://{}", addr)).expect("Is valid url; qed");

            let mut rt = tokio::runtime::Runtime::new().unwrap();
            let executor = rt.executor();
            let client_future = ClientBuilder::<Runtime>::new()
                .set_url(addr.clone())
                .build();
            let client = rt.block_on(client_future).unwrap();

            let stream = rt.block_on(client.subscribe_events()).unwrap();
            let block_events = stream
                .for_each(move |change_set| {
                    change_set
                        .changes
                        .iter()
                        .filter_map(|(_key, data)| {
                            data.as_ref().map(|data| Decode::decode(&mut &data.0[..]))
                        })
                        .for_each(
                            |result: Result<
                                Vec<
                                    srml_system::EventRecord<
                                        <Runtime as System>::Event,
                                        <Runtime as System>::Hash,
                                    >,
                                >,
                                codec::Error,
                            >| {
                                let _ = result.map(|events| {
                                    events.into_iter().for_each(|event| match event.event {
                                        pwc_node_runtime::Event::uniswap(
                                            UniswapEvent::ReserveChanged(1, balance),
                                        ) => {
                                            println!(
                                                "The balance of asset 1 has changed to {:?}",
                                                balance
                                            );
                                            if balance <= 80 {
                                                let signer = AccountKeyring::Bob.pair();
                                                let add_liquidity = ClientBuilder::<Runtime>::new()
                                                    .set_url(addr.clone())
                                                    .build()
                                                    .and_then(move |client| client.xt(signer, None))
                                                    .and_then(move |xt| {
                                                        xt.superfluid(|calls| {
                                                            calls.add_liquidity(1, 2, 20, 0)
                                                        })
                                                        .submit()
                                                    })
                                                    .map(|_| ())
                                                    .map_err(|e| println!("{:?}", e));

                                                println!("add 20 liquidity to asset 1");
                                                executor.spawn(add_liquidity);
                                            } else if balance >= 120 {
                                                let swap = ClientBuilder::<Runtime>::new()
                                                    .set_url(addr.clone())
                                                    .build()
                                                    .and_then(move |client| {
                                                        client.xt(AccountKeyring::Bob.pair(), None)
                                                    })
                                                    .and_then(move |xt| {
                                                        xt.superfluid(|calls| {
                                                            calls.swap_assets_with_exact_output(
                                                                AccountKeyring::Bob.pair().public(),
                                                                0,
                                                                1,
                                                                20,
                                                                20,
                                                            )
                                                        })
                                                        .submit()
                                                    })
                                                    .map(|_| ())
                                                    .map_err(|e| println!("{:?}", e));

                                                println!("swap 20 asset 1 for profit");
                                                executor.spawn(swap);
                                            }
                                        }
                                        _ => {}
                                    })
                                });
                            },
                        );
                    Ok(())
                })
                .map_err(|e| println!("{:?}", e));
            rt.spawn(block_events);
            rt.shutdown_on_idle().wait().unwrap();
        }
        _ => print_usage(&matches),
    }
}

struct Runtime;

impl System for Runtime {
    type Index = <pwc_node_runtime::Runtime as srml_system::Trait>::Index;
    type BlockNumber = <pwc_node_runtime::Runtime as srml_system::Trait>::BlockNumber;
    type Hash = <pwc_node_runtime::Runtime as srml_system::Trait>::Hash;
    type Hashing = <pwc_node_runtime::Runtime as srml_system::Trait>::Hashing;
    type AccountId = <pwc_node_runtime::Runtime as srml_system::Trait>::AccountId;
    type Lookup = <pwc_node_runtime::Runtime as srml_system::Trait>::Lookup;
    type Header = <pwc_node_runtime::Runtime as srml_system::Trait>::Header;
    type Event = <pwc_node_runtime::Runtime as srml_system::Trait>::Event;

    type SignedExtra = (
        srml_system::CheckVersion<pwc_node_runtime::Runtime>,
        srml_system::CheckGenesis<pwc_node_runtime::Runtime>,
        srml_system::CheckEra<pwc_node_runtime::Runtime>,
        srml_system::CheckNonce<pwc_node_runtime::Runtime>,
        srml_system::CheckWeight<pwc_node_runtime::Runtime>,
        srml_balances::TakeFees<pwc_node_runtime::Runtime>,
    );
    fn extra(nonce: Self::Index) -> Self::SignedExtra {
        (
            srml_system::CheckVersion::<pwc_node_runtime::Runtime>::new(),
            srml_system::CheckGenesis::<pwc_node_runtime::Runtime>::new(),
            srml_system::CheckEra::<pwc_node_runtime::Runtime>::from(Era::Immortal),
            srml_system::CheckNonce::<pwc_node_runtime::Runtime>::from(nonce),
            srml_system::CheckWeight::<pwc_node_runtime::Runtime>::new(),
            srml_balances::TakeFees::<pwc_node_runtime::Runtime>::from(0),
        )
    }
}

impl Superfluid for Runtime {
    type Balance = <pwc_node_runtime::Runtime as UniswapTrait>::Balance;
    type AssetId = <pwc_node_runtime::Runtime as UniswapTrait>::AssetId;
}
