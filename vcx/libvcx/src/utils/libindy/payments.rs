use futures::Future;
use indy::payments;
use serde_json::Value;

use std::fmt;
use std::collections::HashMap;

use utils::libindy::wallet::get_wallet_handle;
use utils::libindy::ledger::{libindy_submit_request, libindy_sign_and_submit_request, libindy_sign_request, append_txn_author_agreement_to_request, auth_rule};
use utils::constants::{SUBMIT_SCHEMA_RESPONSE, CREATE_TRANSFER_ACTION};
use settings;
use error::prelude::*;

static DEFAULT_FEES: &str = r#"{"0":0, "1":0, "3":0, "100":0, "101":2, "102":42, "103":0, "104":0, "105":0, "107":0, "108":0, "109":0, "110":0, "111":0, "112":0, "113":2, "114":2, "115":0, "116":0, "117":0, "118":0, "119":0, "10001":0}"#;

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletInfo {
    balance: u64,
    balance_str: String,
    addresses: Vec<AddressInfo>,
}

impl WalletInfo {
    pub fn get_balance(&self) -> u64 {
        self.balance
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddressInfo {
    pub address: String,
    pub balance: u64,
    utxo: Vec<UTXO>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct UTXO {
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(rename = "paymentAddress")]
    recipient: String,
    amount: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Output {
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    recipient: String,
    amount: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<String>,
}

impl fmt::Display for WalletInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match ::serde_json::to_string(&self) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "null"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PaymentTxn {
    pub amount: u64,
    pub credit: bool,
    pub inputs: Vec<String>,
    pub outputs: Vec<Output>,
}

impl PaymentTxn {
    pub fn from_parts(inputs: Vec<String>, outputs: Vec<Output>, amount: u64, credit: bool) -> PaymentTxn {
        PaymentTxn {
            amount,
            credit,
            inputs,
            outputs,
        }
    }
}

pub fn build_test_address(address: &str) -> String {
    format!("pay:{}:{}", ::settings::get_payment_method(), address)
}

pub fn create_address(seed: Option<String>) -> VcxResult<String> {
    trace!("create_address >>> seed: {:?}", seed);

    if settings::indy_mocks_enabled() {
        return Ok(build_test_address("J81AxU9hVHYFtJc"));
    }

    let config = match seed {
        Some(x) => json!({"seed": x}).to_string(),
        None => "{}".to_string(),
    };

    payments::create_payment_address(get_wallet_handle(), settings::get_payment_method().as_str(), &config)
        .wait()
        .map_err(VcxError::from)
}

pub fn sign_with_address(address: &str, message: &[u8]) -> VcxResult<Vec<u8>> {
    trace!("sign_with_address >>> address: {:?}, message: {:?}", address, message);

    if settings::indy_mocks_enabled() {return Ok(Vec::from(message).to_owned()); }

    payments::sign_with_address(get_wallet_handle(), address, message).wait().map_err(VcxError::from)
}

pub fn verify_with_address(address: &str, message: &[u8], signature: &[u8]) -> VcxResult<bool> {
    trace!("sign_with_address >>> address: {:?}, message: {:?}", address, message);

    if settings::indy_mocks_enabled() { return Ok(true); }

    payments::verify_with_address(address, message, signature).wait().map_err(VcxError::from)
}

pub fn get_address_info(address: &str) -> VcxResult<AddressInfo> {
    if settings::indy_mocks_enabled() {
        let utxos = json!(
            [
                {
                    "source": build_test_address("1"),
                    "paymentAddress": build_test_address("zR3GN9lfbCVtHjp"),
                    "amount": 1,
                    "extra": "yqeiv5SisTeUGkw"
                },
                {
                    "source": build_test_address("2"),
                    "paymentAddress": build_test_address("zR3GN9lfbCVtHjp"),
                    "amount": 2,
                    "extra": "Lu1pdm7BuAN2WNi"
                }
            ]
        );

        let utxo: Vec<UTXO> = ::serde_json::from_value(utxos).unwrap();

        return Ok(AddressInfo { address: address.to_string(), balance: _address_balance(&utxo), utxo });
    }

    let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    let (txn, _) = payments::build_get_payment_sources_with_from_request(get_wallet_handle(), Some(&did), address, None)
        .wait()?;

    let response = libindy_sign_and_submit_request(&did, &txn)?;

    let (response, next) = payments::parse_get_payment_sources_with_from_response(settings::get_payment_method().as_str(), &response)
        .wait()?;

    let mut utxo: Vec<UTXO> = ::serde_json::from_str(&response.clone())
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize payment sources response: {}", err)))?;
    let mut next_seqno = next;

    while next_seqno.is_some() {
        let (txn, _) = payments::build_get_payment_sources_with_from_request(get_wallet_handle(), Some(&did), address, next_seqno)
            .wait()?;

        let response = libindy_sign_and_submit_request(&did, &txn)?;

        let (response, next) = payments::parse_get_payment_sources_with_from_response(settings::get_payment_method().as_str(), &response)
            .wait()?;
        let mut res: Vec<UTXO> = ::serde_json::from_str(&response)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize payment sources response: {}", err)))?;
        next_seqno = next;

        utxo.append(&mut res);
    }

    let info = AddressInfo { address: address.to_string(), balance: _address_balance(&utxo), utxo };

    Ok(info)
}

pub fn list_addresses() -> VcxResult<Vec<String>> {
    if settings::indy_mocks_enabled() {
        let addresses = json!([
                build_test_address("9UFgyjuJxi1i1HD"),
                build_test_address("zR3GN9lfbCVtHjp")
        ]);
        return Ok(::serde_json::from_value(addresses).unwrap());
    }

    let addresses = payments::list_payment_addresses(get_wallet_handle())
        .wait()?;

    ::serde_json::from_str(&addresses)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize a list of payment addresses: {}", err)))
}

fn is_valid_address(address: &str, method: &str) -> bool {
    let prefix = format!("pay:{}", method);
    address.starts_with(&prefix)
}

pub fn get_wallet_token_info() -> VcxResult<WalletInfo> {
    trace!("get_wallet_token_info >>>");

    let addresses = list_addresses()?;
    let method = settings::get_config_value(settings::CONFIG_PAYMENT_METHOD)?;

    let mut wallet_info = Vec::new();
    let mut balance = 0;

    for address in addresses.iter() {
        if is_valid_address(&address, &method) {
            debug!("getting address info for {}", address);
            let info = get_address_info(&address)?;

            for utxo in info.utxo.iter() { balance += utxo.amount as u64; }

            wallet_info.push(info);
        } else {
            warn!("payment address {} is not compatible with payment type '{}'", address, method);
        }
    }

    let info = WalletInfo { balance, balance_str: format!("{}", balance), addresses: wallet_info };

    trace!("get_wallet_token_info <<< info: {:?}", info);

    Ok(info)
}

pub fn get_ledger_fees() -> VcxResult<String> {
    trace!("get_ledger_fees >>>");

    if settings::indy_mocks_enabled() { return Ok(DEFAULT_FEES.to_string()); }

    let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    let txn = payments::build_get_txn_fees_req(get_wallet_handle(), Some(&did), settings::get_payment_method().as_str())
        .wait()?;

    let response = libindy_sign_and_submit_request(&did, &txn)?;

    payments::parse_get_txn_fees_response(settings::get_payment_method().as_str(), &response)
        .wait()
        .map_err(VcxError::from)
}

pub fn pay_for_txn(req: &str, txn_action: (&str, &str, &str, Option<&str>, Option<&str>)) -> VcxResult<(Option<PaymentTxn>, String)> {
    debug!("pay_for_txn(req: {}, txn_action: {:?})", req, txn_action);
    if settings::indy_mocks_enabled() {
        let inputs = vec!["pay:null:9UFgyjuJxi1i1HD".to_string()];
        let outputs = serde_json::from_str::<Vec<::utils::libindy::payments::Output>>(r#"[{"amount":1,"extra":null,"recipient":"pay:null:xkIsxem0YNtHrRO"}]"#).unwrap();
        return Ok((Some(PaymentTxn::from_parts(inputs, outputs, 1, false)), SUBMIT_SCHEMA_RESPONSE.to_string()));
    }

    let txn_price = get_action_price(txn_action, None)?;
    if txn_price == 0 {
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;
        let txn_response = libindy_sign_and_submit_request(&did, req)?;
        Ok((None, txn_response))
    } else {
        let (refund, inputs, refund_address) = inputs(txn_price)?;
        let output = outputs(refund, &refund_address, None, None)?;

        let (_fee_response, txn_response) = _submit_fees_request(req, &inputs, &output)?;

        let payment = PaymentTxn::from_parts(inputs, output, txn_price, false);
        Ok((Some(payment), txn_response))
    }
}

fn _serialize_inputs_and_outputs(inputs: &Vec<String>, outputs: &Vec<Output>) -> VcxResult<(String, String)> {
    let inputs = ::serde_json::to_string(inputs)
        .to_vcx(VcxErrorKind::InvalidJson, "Cannot serialize inputs")?;
    let outputs = ::serde_json::to_string(outputs)
        .to_vcx(VcxErrorKind::InvalidJson, "Cannot serialize outputs")?;
    Ok((inputs, outputs))
}

fn _submit_fees_request(req: &str, inputs: &Vec<String>, outputs: &Vec<Output>) -> VcxResult<(String, String)> {
    let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    let (inputs, outputs) = _serialize_inputs_and_outputs(inputs, outputs)?;

    let req = libindy_sign_request(&did, req)?;

    let (req, payment_method) =
        payments::add_request_fees(get_wallet_handle(),
                                   Some(&did),
                                   &req,
                                   &inputs,
                                   &outputs,
                                   None)
            .wait()?;

    let response = libindy_submit_request(&req)?;

    let parsed_response = payments::parse_response_with_fees(&payment_method, &response)
        .wait()
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("Cannot parse response: {}", err)))?;

    Ok((parsed_response, response))
}

pub fn pay_a_payee(price: u64, address: &str) -> VcxResult<(PaymentTxn, String)> {
    trace!("pay_a_payee >>> price: {}, address {}", price, address);
    debug!("sending {} tokens to address {}", price, address);

    let ledger_cost = get_action_price(CREATE_TRANSFER_ACTION, None)?;
    let (remainder, input, refund_address) = inputs(price + ledger_cost)?;
    let outputs = outputs(remainder, &refund_address, Some(address.to_string()), Some(price))?;

    let my_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    if settings::indy_mocks_enabled() {
        let inputs = vec![build_test_address("9UFgyjuJxi1i1HD")];

        let outputs = vec![Output { source: None, recipient: build_test_address("xkIsxem0YNtHrRO"), amount: 1, extra: None}];
        return Ok((PaymentTxn::from_parts(inputs, outputs, 1, false), SUBMIT_SCHEMA_RESPONSE.to_string()));
    }

    let (inputs_json, outputs_json) = _serialize_inputs_and_outputs(&input, &outputs)?;

    let extra = match ::utils::author_agreement::get_txn_author_agreement()? {
        Some(meta) => {
            Some(payments::prepare_extra_with_acceptance_data(None,
                                                              meta.text.as_ref().map(String::as_str),
                                                              meta.version.as_ref().map(String::as_str),
                                                              meta.taa_digest.as_ref().map(String::as_str),
                                                              &meta.acceptance_mechanism_type,
                                                              meta.time_of_acceptance)
                .wait()?)
        }
        None => None
    };

    let (request, _payment_method) =
        payments::build_payment_req(get_wallet_handle(), Some(&my_did), &inputs_json, &outputs_json, extra.as_ref().map(String::as_str))
            .wait()?;

    let result = libindy_submit_request(&request)?;
    let payment = PaymentTxn::from_parts(input, outputs, price, false);
    Ok((payment, result))
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RequestInfo {
    pub price: u64,
    pub requirements: Vec<::serde_json::Value>
}

fn get_request_info(get_auth_rule_resp_json: &str, requester_info_json: &str, fees_json: &str) -> VcxResult<RequestInfo> {
    let req_info = payments::get_request_info(get_auth_rule_resp_json, requester_info_json, fees_json)
        .wait()?;

    ::serde_json::from_str(&req_info)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, err))
}

pub fn get_request_price(action_json: String, requester_info_json: Option<String>) -> VcxResult<u64> {
    let action: auth_rule::Action = ::serde_json::from_str(&action_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize Action: {:?}", err)))?;

    get_action_price((&action.auth_type,
                      &action.auth_action,
                      &action.field,
                      action.old_value.as_ref().map(String::as_str),
                      action.new_value.as_ref().map(String::as_str)),
                     requester_info_json)
}

fn get_action_price(action: (&str, &str, &str, Option<&str>, Option<&str>), requester_info_json: Option<String>) -> VcxResult<u64> {
    let get_auth_rule_resp = match auth_rule::get_action_auth_rule(action) {
        // TODO: Huck to save backward compatibility
        Ok(resp) => resp,
        Err(_) => return Ok(0)
    };

    let ledger_fees = get_ledger_fees()?;

    let requester_info = get_requester_info(requester_info_json)?;

    let req_info = get_request_info(&get_auth_rule_resp, &requester_info, &ledger_fees)?;

    Ok(req_info.price)
}

fn get_requester_info(requester_info_json: Option<String>) -> VcxResult<String> {
    // TODO: THINK better
    let role = match settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_ROLE) {
        Ok(role) => role,
        Err(_) => {
            let role_ = ::utils::libindy::ledger::get_role(&settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?)?;
            settings::set_config_value(settings::CONFIG_SDK_TO_REMOTE_ROLE, &role_);
            role_
        }
    };
    let role = if role == "null" { None } else { Some(role) };

    // TODO: think about better way
    let res = match requester_info_json {
        Some(requester_info) => requester_info,
        None => json!({
            "role": role,
            "sig_count": 1,
            "is_owner": true,
            "is_off_ledger_signature": false,
        }).to_string()
    };
    Ok(res)
}

fn _address_balance(address: &Vec<UTXO>) -> u64 {
    address.iter().fold(0, |balance, utxo| balance + utxo.amount)
}

pub fn inputs(cost: u64) -> VcxResult<(u64, Vec<String>, String)> {
    let mut inputs: Vec<String> = Vec::new();
    let mut balance = 0;
    let mut refund_address = String::new();

    let wallet_info: WalletInfo = get_wallet_token_info()?;

    if wallet_info.balance < cost {
        warn!("not enough tokens in wallet to pay: balance: {}, cost: {}", wallet_info.balance, cost);
        return Err(VcxError::from_msg(VcxErrorKind::InsufficientTokenAmount, format!("Not enough tokens in wallet to pay: balance: {}, cost: {}", wallet_info.balance, cost)));
    }

    // Todo: explore 'smarter' ways of selecting utxos ie bitcoin algorithms etc
    'outer: for address in wallet_info.addresses.iter() {
        refund_address = address.address.clone();
        'inner: for utxo in address.utxo.iter() {
            if balance < cost {
                inputs.push(utxo.source.clone().ok_or(VcxErrorKind::InsufficientTokenAmount)?.to_string());
                balance += utxo.amount;
            } else { break 'outer }
        }
    }

    let remainder = balance - cost;

    Ok((remainder, inputs, refund_address))
}

pub fn outputs(remainder: u64, refund_address: &str, payee_address: Option<String>, payee_amount: Option<u64>) -> VcxResult<Vec<Output>> {
    // In the future we might provide a way for users to specify multiple output address for their remainder tokens
    // As of now, we only handle one output address which we create

    let mut outputs = Vec::new();
    if remainder > 0 {
        outputs.push(Output { source: None, recipient: refund_address.to_string(), amount: remainder, extra: None });
    }

    if let Some(address) = payee_address {
        outputs.push(Output { source: None, recipient: address, amount: payee_amount.unwrap_or(0), extra: None });
    }

    Ok(outputs)
}

// This is used for testing purposes only!!!
pub fn mint_tokens_and_set_fees(number_of_addresses: Option<u32>, tokens_per_address: Option<u64>, fees: Option<String>, seed: Option<String>) -> VcxResult<()> {
    trace!("mint_tokens_and_set_fees >>> number_of_addresses: {:?}, tokens_per_address: {:?}, fees: {:?}, seed: {:?}",
           number_of_addresses, tokens_per_address, fees, seed);

    let did_1 = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

    let fees = if fees.is_some() {
        fees.as_ref().map(String::as_str)
    } else {
        None
    };

    let (did_2, _) = add_new_did(Some("TRUSTEE"));
    let (did_3, _) = add_new_did(Some("TRUSTEE"));
    let (did_4, _) = add_new_did(Some("TRUSTEE"));

    let number_of_addresses = number_of_addresses.unwrap_or(1);

    if number_of_addresses > 0 {
        let tokens_per_address: u64 = tokens_per_address.unwrap_or(50_000_000_000);
        let mut addresses = Vec::new();

        for _n in 0..number_of_addresses {
            addresses.push(create_address(seed.clone()).unwrap())
        }

        let mint: Vec<Value> = addresses.clone().into_iter().enumerate().map(|(_i, payment_address)|
            json!( { "recipient": payment_address, "amount": tokens_per_address } )
        ).collect();
        let outputs = serde_json::to_string(&mint).unwrap();
        let (req, _) = payments::build_mint_req(get_wallet_handle(), Some(&did_1), &outputs, None).wait().unwrap();

        let sign1 = ::utils::libindy::ledger::multisign_request(&did_1, &req).unwrap();
        let sign2 = ::utils::libindy::ledger::multisign_request(&did_2, &sign1).unwrap();
        let sign3 = ::utils::libindy::ledger::multisign_request(&did_3, &sign2).unwrap();
        let sign4 = ::utils::libindy::ledger::multisign_request(&did_4, &sign3).unwrap();

        match ::utils::libindy::ledger::libindy_submit_request(&sign4) {
            Ok(_) => (),
            Err(x) => println!("failure minting tokens: {}", x),
        };
    }

    if let Some(fees_) = fees {
        let txn = payments::build_set_txn_fees_req(get_wallet_handle(), Some(&did_1), settings::get_payment_method().as_str(), fees_)
            .wait()?;

        let sign1 = ::utils::libindy::ledger::multisign_request(&did_1, &txn).unwrap();
        let sign2 = ::utils::libindy::ledger::multisign_request(&did_2, &sign1).unwrap();
        let sign3 = ::utils::libindy::ledger::multisign_request(&did_3, &sign2).unwrap();
        let sign4 = ::utils::libindy::ledger::multisign_request(&did_4, &sign3).unwrap();

        ::utils::libindy::ledger::libindy_submit_request(&sign4).unwrap();

        let txn_fees: HashMap<String, String> =
            ::serde_json::from_str::<HashMap<String, u64>>(fees_).unwrap()
                .iter_mut()
                .map(|(k, _v)| (k.to_string(), k.to_string()))
                .collect();

        auth_rule::set_actions_fee_aliases(&did_1, &json!(txn_fees).to_string()).unwrap();
    }

    Ok(())
}

pub fn add_new_did(role: Option<&str>) -> (String, String) {
    use indy::ledger;

    let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

    let (did, verkey) = ::utils::libindy::signus::create_and_store_my_did(None, None).unwrap();
    let mut req_nym = ledger::build_nym_request(&institution_did, &did, Some(&verkey), None, role).wait().unwrap();

    req_nym = append_txn_author_agreement_to_request(&req_nym).unwrap();

    ::utils::libindy::ledger::libindy_sign_and_submit_request(&institution_did, &req_nym).unwrap();
    (did, verkey)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use utils::devsetup::*;

    static ZERO_FEES: &str = r#"{"0":0, "1":0, "101":0, "10001":0, "102":0, "103":0, "104":0, "105":0, "107":0, "108":0, "109":0, "110":0, "111":0, "112":0, "113":0, "114":0, "115":0, "116":0, "117":0, "118":0, "119":0}"#;

    pub fn token_setup(number_of_addresses: Option<u32>, tokens_per_address: Option<u64>, use_zero_fees: bool) {
        let fees = if use_zero_fees { ZERO_FEES } else { DEFAULT_FEES };
        mint_tokens_and_set_fees(number_of_addresses, tokens_per_address, Some(fees.to_string()), None).unwrap();
    }

    #[allow(dead_code)]
    fn get_my_balance() -> u64 {
        let info: WalletInfo = get_wallet_token_info().unwrap();
        info.balance
    }

    #[test]
    fn test_create_address() {
        let _setup = SetupMocks::init();

        create_address(None).unwrap();
    }


    #[test]
    fn test_sign_with_address() {
        let _setup = SetupMocks::init();

        let res = sign_with_address("test", &[1, 2, 3]).unwrap();
        assert_eq!(res, vec![1, 2, 3])
    }

    #[test]
    fn test_verify_with_address() {
        let _setup = SetupMocks::init();

        let res = verify_with_address("test", &[1, 2, 3], &[1, 2, 3]).unwrap();
        assert!(res)
    }

    #[test]
    fn test_get_addresses() {
        let _setup = SetupMocks::init();

        create_address(None).unwrap();
        let _addresses = list_addresses().unwrap();
    }

    #[test]
    fn test_get_wallet_token_info() {
        let _setup = SetupMocks::init();

        create_address(None).unwrap();
        let balance = get_wallet_token_info().unwrap().to_string();

        let expected_balance = json!({
            "balance":6,
            "balance_str":"6",
            "addresses":[
                {
                    "address": build_test_address("9UFgyjuJxi1i1HD"),
                    "balance":3,
                    "utxo":[
                        {
                            "source": build_test_address("1"),
                            "paymentAddress":build_test_address("zR3GN9lfbCVtHjp"),
                            "amount":1,
                            "extra":"yqeiv5SisTeUGkw"
                        },
                        {
                            "source":build_test_address("2"),
                            "paymentAddress":build_test_address("zR3GN9lfbCVtHjp"),
                            "amount":2,
                            "extra":"Lu1pdm7BuAN2WNi"
                        }
                    ]
                },
                {
                    "address":build_test_address("zR3GN9lfbCVtHjp"),
                    "balance":3,
                    "utxo":[
                        {
                            "source":build_test_address("1"),
                            "paymentAddress":build_test_address("zR3GN9lfbCVtHjp"),
                            "amount":1,
                            "extra":"yqeiv5SisTeUGkw"
                        },
                        {
                            "source":build_test_address("2"),
                            "paymentAddress":build_test_address("zR3GN9lfbCVtHjp"),
                            "amount":2,
                            "extra":"Lu1pdm7BuAN2WNi"
                        }
                    ]
                }
            ]
        });

        let balance: serde_json::Value = serde_json::from_str(&balance).unwrap();

        assert_eq!(balance, expected_balance);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_get_wallet_token_info_real() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let wallet_info = get_wallet_token_info().unwrap();
        assert_eq!(wallet_info.balance, 50000000000);
    }

    #[test]
    fn test_get_ledger_fees() {
        let _setup = SetupMocks::init();

        let fees = get_ledger_fees().unwrap();
        assert!(fees.contains(r#""101":2"#));
        assert!(fees.contains(r#""1":0"#));
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_get_ledger_fees_real() {
        let _setup = SetupLibraryWalletPool::init();

        let fees = get_ledger_fees().unwrap();
        assert!(fees.contains(r#""101":2"#));
        assert!(fees.contains(r#""1":0"#));
    }

    #[test]
    fn test_address_balance() {
        let _setup = SetupDefaults::init();

        let addresses = vec![
            UTXO { source: Some(build_test_address("2")), recipient: build_test_address("J81AxU9hVHYFtJc"), amount: 2, extra: Some("abcde".to_string()) },
            UTXO { source: Some(build_test_address("3")), recipient: build_test_address("J81AxU9hVHYFtJc"), amount: 3, extra: Some("bcdef".to_string()) }
        ];

        assert_eq!(_address_balance(&addresses), 5);
    }

    #[test]
    fn test_inputs() {
        let _setup = SetupMocks::init();

        let pay_addr_1 = build_test_address("1");
        let pay_addr_2 = build_test_address("2");

        // Success - Exact amount
        let expected_inputs = vec![pay_addr_1.clone(), pay_addr_2.clone(), pay_addr_1.clone(), pay_addr_2.clone()];
        assert_eq!(inputs(6).unwrap(), (0, expected_inputs, build_test_address("zR3GN9lfbCVtHjp")));

        // Success - utxo with remainder tokens
        let expected_inputs = vec![pay_addr_1.clone(), pay_addr_2.clone(), pay_addr_1.clone(), pay_addr_2.clone()];
        assert_eq!(inputs(5).unwrap(), (1, expected_inputs, build_test_address("zR3GN9lfbCVtHjp")));

        // Success - requesting amount that partial address (1 of 2 utxos) can satisfy
        let expected_inputs = vec![pay_addr_1.clone()];
        assert_eq!(inputs(1).unwrap(), (0, expected_inputs, build_test_address("9UFgyjuJxi1i1HD")));

        // Err - request more than wallet contains
        assert_eq!(inputs(7).err().unwrap().kind(), VcxErrorKind::InsufficientTokenAmount);
    }

    #[test]
    fn test_gen_outputs_for_txn_fees() {
        let _setup = SetupMocks::init();

        let mut cost = 5;
        let (remainder, _, refund_address) = inputs(cost).unwrap();
        let mut expected_output: Vec<Output> = ::serde_json::from_str(&format!(r#"[{{"amount":1,"recipient":"{}"}}]"#, refund_address)).unwrap();
        assert_eq!(outputs(remainder, &refund_address, None, None).unwrap(), expected_output);

        // No remainder so don't create an address in outputs
        cost = 6;
        expected_output = vec![];
        let (remainder, _, refund_address) = inputs(cost).unwrap();
        assert_eq!(remainder, 0);
        assert_eq!(outputs(remainder, &refund_address, None, None).unwrap(), expected_output);
    }

    #[test]
    fn test_gen_outputs_for_transfer_of_tokens() {
        let _setup = SetupMocks::init();

        let payee_amount = 11;
        let payee_address = build_test_address("payee_address");
        let refund_address = build_test_address("refund_address");
        let expected_output: Vec<Output> = ::serde_json::from_str(&format!(r#"[{{"amount":4,"recipient":"{}"}},{{"amount":11,"recipient":"{}"}}]"#, refund_address, payee_address)).unwrap();
        assert_eq!(outputs(4, refund_address.as_str(), Some(payee_address), Some(payee_amount)).unwrap(), expected_output);
    }

    #[test]
    fn test_get_txn_cost() {
        let _setup = SetupMocks::init();

        assert_eq!(get_action_price(::utils::constants::CREATE_SCHEMA_ACTION, None).unwrap(), 2);
        assert_eq!(get_action_price(::utils::constants::CREATE_CRED_DEF_ACTION, None).unwrap(), 42);

        let unknown_action = ("unknown txn", "ADD", "*", None, Some("*"));
        assert_eq!(get_action_price(unknown_action, None).unwrap(), 0);
    }

    #[test]
    fn test_pay_for_txn() {
        let _setup = SetupMocks::init();

        // Schema
        let create_schema_req = ::utils::constants::SCHEMA_CREATE_JSON.to_string();
        let (_payment, response) = pay_for_txn(&create_schema_req, ::utils::constants::CREATE_SCHEMA_ACTION).unwrap();
        assert_eq!(response, SUBMIT_SCHEMA_RESPONSE.to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_pay_for_txn_real() {
        let _setup = SetupLibraryWalletPool::init();

        let (_, schema_json) = ::utils::libindy::anoncreds::tests::create_schema(::utils::constants::DEFAULT_SCHEMA_ATTRS);
        let create_schema_req = ::utils::libindy::anoncreds::tests::create_schema_req(&schema_json);
        let start_wallet = get_wallet_token_info().unwrap();

        let (payment, _response) = pay_for_txn(&create_schema_req, ::utils::constants::CREATE_SCHEMA_ACTION).unwrap();

        let end_wallet = get_wallet_token_info().unwrap();

        let payment = payment.unwrap();
        assert_eq!(payment.amount, 2);
        assert_eq!(payment.outputs.len(), 1);
        assert_eq!(start_wallet.balance - 2, end_wallet.balance);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_pay_for_txn_fails_with_insufficient_tokens_in_wallet() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        mint_tokens_and_set_fees(Some(0), Some(0), Some(r#"{"101":50000000001}"#.to_string()), None).unwrap();

        let (_, schema_json) = ::utils::libindy::anoncreds::tests::create_schema(::utils::constants::DEFAULT_SCHEMA_ATTRS);
        let create_schema_req = ::utils::libindy::anoncreds::tests::create_schema_req(&schema_json);

        let rc = pay_for_txn(&create_schema_req, ::utils::constants::CREATE_SCHEMA_ACTION);

        assert!(rc.is_err());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_build_payment_request() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let payment_address = build_test_address("2ZrAm5Jc3sP4NAXMQbaWzDxEa12xxJW3VgWjbbPtMPQCoznJyS");
        let payment_address = payment_address.as_str();

        let price = get_my_balance();
        let result_from_paying = pay_a_payee(price, payment_address);
        assert!(result_from_paying.is_ok());
        assert_eq!(get_my_balance(), 0);
        mint_tokens_and_set_fees(None, None, None, None).unwrap();
        assert_eq!(get_my_balance(), 50000000000);

        let price = get_my_balance() - 5;
        let result_from_paying = pay_a_payee(price, payment_address);
        assert!(result_from_paying.is_ok());
        assert_eq!(get_my_balance(), 5);

        let price = get_my_balance() + 5;
        let result_from_paying = pay_a_payee(price, payment_address);
        assert_eq!(result_from_paying.err().unwrap().kind(), VcxErrorKind::InsufficientTokenAmount);
        assert_eq!(get_my_balance(), 5);
    }

    // this test if failing to to both changes in error codes being produced
    // by master libindy and how wallets are deleted.
    #[cfg(feature = "pool_tests")]
    #[test]
    #[ignore]
    fn test_build_payment_request_bogus_payment_method() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let payment_address = "pay:bogus:123";
        let result_from_paying = pay_a_payee(0, payment_address);

        assert!(result_from_paying.is_err());
        assert_eq!(result_from_paying.err().unwrap().kind(), VcxErrorKind::LibndyError(100)); // TODO: FIXME
    }

    #[cfg(feature = "pool_tests")]
    #[ignore] // FIXME: there are no auth rules for XFER transaction on the ledger.
    #[test]
    fn test_fees_transferring_tokens() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let payment_address = build_test_address("2ZrAm5Jc3sP4NAXMQbaWzDxEa12xxJW3VgWjbbPtMPQCoznJyS");
        let payment_address = payment_address.as_str();

        let initial_wallet_balance = 100000000000;
        let transfer_fee = 5;
        let ledger_fees = json!({"10001": transfer_fee}).to_string();
        mint_tokens_and_set_fees(None, None, Some(ledger_fees), None).unwrap();
        assert_eq!(get_my_balance(), initial_wallet_balance);
        assert_eq!(get_action_price(CREATE_TRANSFER_ACTION, None).unwrap(), transfer_fee);

        // Transfer everything besides 50. Remaining balance will be 50 - ledger fees
        let balance_after_transfer = 50;
        let price = get_my_balance() - balance_after_transfer;
        let result_from_paying = pay_a_payee(price, payment_address);
        assert!(result_from_paying.is_ok());
        assert_eq!(get_my_balance(), balance_after_transfer - transfer_fee);

        // Has tokens but not enough for ledger fee
        let not_enough_for_ledger_fee = transfer_fee - 1;
        let price = get_my_balance() - not_enough_for_ledger_fee;
        assert!(price > 0);
        let result_from_paying = pay_a_payee(price, payment_address);
        assert_eq!(result_from_paying.err().unwrap().kind(), VcxErrorKind::InsufficientTokenAmount);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_submit_fees_with_insufficient_tokens_on_ledger() {
        let _setup = SetupLibraryWalletPool::init();

        let (_, schema_json) = ::utils::libindy::anoncreds::tests::create_schema(::utils::constants::DEFAULT_SCHEMA_ATTRS);
        let req = ::utils::libindy::anoncreds::tests::create_schema_req(&schema_json);
        let (remainder, inputs, refund_address) = inputs(2).unwrap();
        let output = outputs(remainder, &refund_address, None, None).unwrap();
        let start_wallet = get_wallet_token_info().unwrap();

        _submit_fees_request(&req, &inputs, &output).unwrap();

        let end_wallet = get_wallet_token_info().unwrap();
        assert_eq!(start_wallet.balance - 2, end_wallet.balance);

        let _rc = _submit_fees_request(&req, &inputs, &output);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_pay_for_txn_with_empty_outputs_success() {
        let _setup = SetupLibraryWalletPool::init();

        let (_, schema_json) = ::utils::libindy::anoncreds::tests::create_schema(::utils::constants::DEFAULT_SCHEMA_ATTRS);
        let req = ::utils::libindy::anoncreds::tests::create_schema_req(&schema_json);

        let cost = get_action_price(::utils::constants::CREATE_SCHEMA_ACTION, None).unwrap();
        let start_wallet = get_wallet_token_info().unwrap();
        let remaining_balance = start_wallet.balance - cost;
        let (remainder, inputs, refund_address) = inputs(cost).unwrap();
        assert_eq!(remainder, remaining_balance);

        let output = outputs(remainder, &refund_address, None, None).unwrap();

        let _rc = _submit_fees_request(&req, &inputs, &output).unwrap();
        let end_wallet = get_wallet_token_info().unwrap();

        assert_eq!(end_wallet.balance, remaining_balance);
    }

    #[test]
    fn test_wallet_info_to_string() {
        let _setup = SetupDefaults::init();

        let wallet_info = WalletInfo {
            balance: 12345,
            addresses: Vec::new(),
            balance_str: "12345".to_string(),
        };
        assert_eq!(wallet_info.to_string(), r#"{"balance":12345,"balance_str":"12345","addresses":[]}"#.to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_custom_mint_tokens() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        //50000000000 comes from setup_ledger_env
        token_setup(Some(4), Some(1430000), false);

        let start_wallet = get_wallet_token_info().unwrap();
        assert_eq!(start_wallet.balance, 50005720000);
    }

    #[ignore] // Test only works when fees are null
    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_empty_fees() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let fees = get_ledger_fees().unwrap();
        println!("fees: {}", fees);
        ::utils::libindy::anoncreds::tests::create_and_write_test_schema(::utils::constants::DEFAULT_SCHEMA_ATTRS);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_zero_fees() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        mint_tokens_and_set_fees(Some(0), Some(0), Some("{\"101\":0, \"102\":0}".to_string()), None).unwrap();
        let fees = get_ledger_fees().unwrap();
        println!("fees: {}", fees);
        ::utils::libindy::anoncreds::tests::create_and_write_test_schema(::utils::constants::DEFAULT_SCHEMA_ATTRS);
    }

    fn _action() -> String {
        json!({
            "auth_type":"101",
            "auth_action":"ADD",
            "new_value":"0",
            "field":"role"
        }).to_string()
    }

    #[test]
    fn get_action_price_for_requester_match_to_constraint() {
        let _setup = SetupMocks::init();

        let requester_info = json!({
            "role": "0",
            "need_to_be_owner":false,
            "sig_count":1,
        }).to_string();

        let price = get_request_price(_action(), Some(requester_info)).unwrap();
        assert_eq!(2, price);
    }

    #[test]
    fn get_action_price_for_requester_not_match_to_constraint() {
        let _setup = SetupMocks::init();

        let action_json = _action();

        let requester_info = json!({
            "role": "101",
            "need_to_be_owner":false,
            "sig_count":1,
        }).to_string();

        let res = get_request_price(action_json, Some(requester_info));
        assert!(res.is_err());
    }
}
