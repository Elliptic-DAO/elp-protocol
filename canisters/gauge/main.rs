use candid::candid_method;
use candid::{Nat, Principal};
use ic_canisters_http_types as http;
use ic_cdk_macros::{init, post_upgrade, query, update};
use std::io::Write;

use ic_canisters_http_types::HttpResponseBuilder;
use ic_icrc1_client_cdk::CdkRuntime;
use ic_icrc1_client_cdk::ICRC1Client;
use ic_stable_structures::storable::Blob;
use ic_stable_structures::StableBTreeMap;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory as VM},
    DefaultMemoryImpl, RestrictedMemory as RM, StableCell, Storable,
};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::TransferArg;
use icrc_ledger_types::icrc1::transfer::TransferError;
use std::borrow::Borrow;
use std::borrow::Cow;
use std::cell::RefCell;

const FIFTY_ICP: u64 = 50 * 100_000_000;

/// Mint an amount of ICP to an Account.
async fn mint(amount: u64, to: Account) -> Result<u64, TransferError> {
    let ledger_canister_id = CONFIG_CELL.with(|p| p.borrow().get().ledger_id);

    let client = ICRC1Client {
        runtime: CdkRuntime,
        ledger_canister_id,
    };
    client
        .transfer(TransferArg {
            from_subaccount: None,
            to,
            fee: None,
            created_at_time: None,
            memo: None,
            amount: Nat::from(amount),
        })
        .await
        .unwrap()
}

fn main() {}

#[init]
#[candid_method(init)]
fn init(maintainer: Principal, ledger_id: Principal) {
    CONFIG_CELL.with(move |cell| {
        cell.borrow_mut()
            .set(Cbor(Config {
                api_key: Default::default(),
                maintainer,
                ledger_id,
            }))
            .expect("failed to initialize the config");
    })
}

#[post_upgrade]
fn post_upgrade(maintainer: Principal, ledger_id: Principal) {
    CONFIG_CELL.with(|cell| {
        let mut config = cell.borrow().get().clone();
        config.maintainer = maintainer;
        config.ledger_id = ledger_id;
        cell.borrow_mut()
            .set(config)
            .expect("failed to update the config cell");
    })
}

#[candid_method(update)]
#[update]
async fn register_user(api_key: String) -> Result<u64, TransferError> {
    let current_key = CONFIG_CELL.with(|p| p.borrow().get().api_key.clone());
    assert!(api_key == current_key);
    let caller = ic_cdk::caller();
    let caller_bytes = Blob::<29>::try_from(caller.as_slice()).unwrap();
    match SUSERS.with(|p| p.borrow().get(&caller_bytes)) {
        Some(current_value) => {
            SUSERS.with(|p| p.borrow_mut().insert(caller_bytes, current_value + 1));
        }
        None => {
            SUSERS.with(|p| p.borrow_mut().insert(caller_bytes, 1));
        }
    }
    mint(
        FIFTY_ICP,
        Account {
            owner: caller,
            subaccount: None,
        },
    )
    .await
}

#[candid_method(update)]
#[update]
async fn set_key(new_key: String) {
    let caller = ic_cdk::caller();
    let maintainer = CONFIG_CELL.with(|p| p.borrow().get().maintainer);
    assert!(caller == maintainer);
    CONFIG_CELL.with(|cell| {
        let mut config = cell.borrow().get().clone();
        config.api_key = new_key;
        cell.borrow_mut()
            .set(config)
            .expect("failed to encode config");
    });
}

fn with_utf8_buffer(f: impl FnOnce(&mut Vec<u8>)) -> String {
    let mut buf = Vec::new();
    f(&mut buf);
    String::from_utf8(buf).unwrap()
}

fn build_table() -> String {
    with_utf8_buffer(|buf| {
        SUSERS.with(|p| {
            for (principal, counter) in p.borrow().iter() {
                writeln!(
                    buf,
                    "<tr><td>{}</td><td>{}</td></tr>",
                    Principal::from_slice(principal.as_slice()),
                    counter
                )
                .unwrap();
            }
        })
    })
}

pub fn build_dashboard() -> Vec<u8> {
    format!(
        "
        <!DOCTYPE html>
        <html lang=\"en\">
            <head>
                <title>Gauge Dashboard</title>
            </head>
            <body>
                <h3>Elliptic Protocol Beta Gauge Dashboard</h3>
                <table>
                    <thead>
                        <tr>
                            <td>Principal</td>
                            <td>Connections counter</td>
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
            </body>
        </html>
    ",
        build_table()
    )
    .into_bytes()
}

#[query]
#[candid_method(query)]
fn http_request(req: http::HttpRequest) -> http::HttpResponse {
    if req.path() == "/dashboard" {
        let dashboard: Vec<u8> = build_dashboard();
        HttpResponseBuilder::ok()
            .header("Content-Type", "text/html; charset=utf-8")
            .with_body_and_content_length(dashboard)
            .build()
    } else if req.path() == "/metrics" {
        let mut writer =
            ic_metrics_encoder::MetricsEncoder::new(vec![], ic_cdk::api::time() as i64 / 1_000_000);
        writer
            .encode_gauge(
                "cycle_balance",
                ic_cdk::api::canister_balance128() as f64,
                "The canister cycle balance.",
            )
            .unwrap();
        writer
            .encode_gauge(
                "stable_memory_bytes",
                ic_cdk::api::stable::stable_size() as f64 * 65536.0,
                "Size of the stable memory allocated by this canister.",
            )
            .unwrap();
        writer
            .encode_gauge(
                "user_count",
                SUSERS.with(|p| p.borrow().len()) as f64,
                "Size of the stable memory allocated by this canister.",
            )
            .unwrap();
        http::HttpResponseBuilder::ok()
            .header("Content-Type", "text/plain; version=0.0.4")
            .with_body_and_content_length(writer.into_inner())
            .build()
    } else {
        http::HttpResponseBuilder::not_found().build()
    }
}

#[derive(Clone, PartialEq, Eq, serde::Serialize, candid::Deserialize)]
struct Config {
    api_key: String,
    maintainer: Principal,
    ledger_id: Principal,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: "".to_string(),
            maintainer: Principal::management_canister(),
            ledger_id: Principal::management_canister(),
        }
    }
}

const METADATA_PAGES: u64 = 16; // 16 pages ~ 2Mb
const USERS: MemoryId = MemoryId::new(0);

type RestrictedMemory = RM<DefaultMemoryImpl>;
type VirtualMemory = VM<RestrictedMemory>;

#[derive(Default, Clone, PartialEq, Eq)]
struct Cbor<T>(pub T);

impl<T> std::ops::Deref for Cbor<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Cbor<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Storable for Cbor<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = vec![];
        ciborium::ser::into_writer(&self.0, &mut buf).unwrap();
        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(
            ciborium::de::from_reader(bytes.as_ref()).unwrap_or_else(|e| {
                panic!(
                    "failed to decode CBOR {}: {}",
                    hex::encode(bytes.as_ref()),
                    e
                )
            }),
        )
    }
}

thread_local! {
    static MEMORY_MANAGER: MemoryManager<RestrictedMemory> =
        MemoryManager::init(
            RestrictedMemory::new(
                DefaultMemoryImpl::default(),
                METADATA_PAGES..u64::MAX/65536 - 1
            ));

    static CONFIG_CELL: RefCell<StableCell<Cbor<Config>, RestrictedMemory>> = RefCell::new(
        StableCell::init(
            RestrictedMemory::new(
                DefaultMemoryImpl::default(),
                0..METADATA_PAGES
            ),
            Cbor(Config::default()),
        ).expect("failed to initialize the config cell")
    );

    static SUSERS: RefCell<StableBTreeMap<Blob<29>, u64, VirtualMemory>> =
    MEMORY_MANAGER.with(|mm| {
      RefCell::new(StableBTreeMap::init(mm.borrow().get(USERS)))
    });

}

#[test]
fn check_candid_interface_compatibility() {
    use candid::utils::{service_compatible, CandidSource};

    candid::export_service!();

    let new_interface = __export_service();

    // check the public interface against the actual one
    let old_interface =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("gauge.did");

    service_compatible(
        CandidSource::Text(&new_interface),
        CandidSource::File(old_interface.as_path()),
    )
    .unwrap();
}
