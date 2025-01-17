#![cfg(feature = "abigen")]
//! Test cases to validate the `abigen!` macro
use ethers_contract::{abigen, AbiDecode, AbiEncode, EthEvent};
use ethers_core::abi::{Address, Tokenizable};
use ethers_core::types::U256;
use ethers_providers::Provider;
use std::sync::Arc;

#[test]
fn can_gen_human_readable() {
    abigen!(
        SimpleContract,
        r#"[
        event ValueChanged(address indexed author, string oldValue, string newValue)
    ]"#,
        event_derives(serde::Deserialize, serde::Serialize)
    );
    assert_eq!("ValueChanged", ValueChangedFilter::name());
    assert_eq!(
        "ValueChanged(address,string,string)",
        ValueChangedFilter::abi_signature()
    );
}

#[test]
fn can_gen_human_readable_multiple() {
    abigen!(
        SimpleContract1,
        r#"[
        event ValueChanged1(address indexed author, string oldValue, string newValue)
    ]"#,
        event_derives(serde::Deserialize, serde::Serialize);

        SimpleContract2,
        r#"[
        event ValueChanged2(address indexed author, string oldValue, string newValue)
    ]"#,
        event_derives(serde::Deserialize, serde::Serialize)
    );
    assert_eq!("ValueChanged1", ValueChanged1Filter::name());
    assert_eq!(
        "ValueChanged1(address,string,string)",
        ValueChanged1Filter::abi_signature()
    );
    assert_eq!("ValueChanged2", ValueChanged2Filter::name());
    assert_eq!(
        "ValueChanged2(address,string,string)",
        ValueChanged2Filter::abi_signature()
    );
}

#[test]
fn can_gen_structs_readable() {
    abigen!(
        SimpleContract,
        r#"[
        struct Value {address addr; string value;}
        struct Addresses {address[] addr; string s;}
        event ValueChanged(Value indexed old, Value newValue, Addresses _a)
    ]"#,
        event_derives(serde::Deserialize, serde::Serialize)
    );
    let value = Addresses {
        addr: vec!["eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap()],
        s: "hello".to_string(),
    };
    let token = value.clone().into_token();
    assert_eq!(value, Addresses::from_token(token).unwrap());

    assert_eq!("ValueChanged", ValueChangedFilter::name());
    assert_eq!(
        "ValueChanged((address,string),(address,string),(address[],string))",
        ValueChangedFilter::abi_signature()
    );
}

#[test]
fn can_gen_structs_with_arrays_readable() {
    abigen!(
        SimpleContract,
        r#"[
        struct Value {address addr; string value;}
        struct Addresses {address[] addr; string s;}
        event ValueChanged(Value indexed old, Value newValue, Addresses[] _a)
    ]"#,
        event_derives(serde::Deserialize, serde::Serialize)
    );
    assert_eq!(
        "ValueChanged((address,string),(address,string),(address[],string)[])",
        ValueChangedFilter::abi_signature()
    );
}

fn assert_tokenizeable<T: Tokenizable>() {}

#[test]
fn can_generate_internal_structs() {
    abigen!(
        VerifierContract,
        "ethers-contract/tests/solidity-contracts/verifier_abi.json",
        event_derives(serde::Deserialize, serde::Serialize)
    );
    assert_tokenizeable::<VerifyingKey>();
    assert_tokenizeable::<G1Point>();
    assert_tokenizeable::<G2Point>();
}

#[test]
fn can_generate_internal_structs_multiple() {
    // NOTE: nesting here is necessary due to how tests are structured...
    use contract::*;
    mod contract {
        use super::*;
        abigen!(
            VerifierContract,
            "ethers-contract/tests/solidity-contracts/verifier_abi.json",
            event_derives(serde::Deserialize, serde::Serialize);

            MyOtherVerifierContract,
            "ethers-contract/tests/solidity-contracts/verifier_abi.json",
            event_derives(serde::Deserialize, serde::Serialize);
        );
    }
    assert_tokenizeable::<VerifyingKey>();
    assert_tokenizeable::<G1Point>();
    assert_tokenizeable::<G2Point>();

    let (provider, _) = Provider::mocked();
    let client = Arc::new(provider);

    let g1 = G1Point {
        x: U256::zero(),
        y: U256::zero(),
    };
    let g2 = G2Point {
        x: [U256::zero(), U256::zero()],
        y: [U256::zero(), U256::zero()],
    };
    let vk = VerifyingKey {
        alfa_1: g1.clone(),
        beta_2: g2.clone(),
        gamma_2: g2.clone(),
        delta_2: g2.clone(),
        ic: vec![g1.clone()],
    };
    let proof = Proof {
        a: g1.clone(),
        b: g2,
        c: g1,
    };

    // ensure both contracts use the same types
    let contract = VerifierContract::new(Address::zero(), client.clone());
    let _ = contract.verify(vec![], proof.clone(), vk.clone());
    let contract = MyOtherVerifierContract::new(Address::zero(), client);
    let _ = contract.verify(vec![], proof, vk);
}

#[test]
fn can_gen_human_readable_with_structs() {
    abigen!(
        SimpleContract,
        r#"[
        struct Foo { uint256 x; }
        function foo(Foo memory x)
        function bar(uint256 x, uint256 y, address addr)
        yeet(uint256,uint256,address)
    ]"#,
        event_derives(serde::Deserialize, serde::Serialize)
    );
    assert_tokenizeable::<Foo>();

    let (client, _mock) = Provider::mocked();
    let contract = SimpleContract::new(Address::default(), Arc::new(client));
    let f = Foo { x: 100u64.into() };
    let _ = contract.foo(f);

    let call = BarCall {
        x: 1u64.into(),
        y: 0u64.into(),
        addr: Address::random(),
    };
    let encoded_call = contract.encode("bar", (call.x, call.y, call.addr)).unwrap();
    assert_eq!(encoded_call, call.clone().encode().unwrap());
    let decoded_call = BarCall::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(call, decoded_call);

    let contract_call = SimpleContractCalls::Bar(call);
    let decoded_enum = SimpleContractCalls::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(contract_call, decoded_enum);
    assert_eq!(encoded_call, contract_call.encode().unwrap());

    let call = YeetCall(1u64.into(), 0u64.into(), Address::zero());
    let encoded_call = contract.encode("yeet", (call.0, call.1, call.2)).unwrap();
    assert_eq!(encoded_call, call.clone().encode().unwrap());
    let decoded_call = YeetCall::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(call, decoded_call);

    let contract_call = SimpleContractCalls::Yeet(call.clone());
    let decoded_enum = SimpleContractCalls::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(contract_call, decoded_enum);
    assert_eq!(contract_call, call.into());
    assert_eq!(encoded_call, contract_call.encode().unwrap());
}

#[test]
fn can_handle_overloaded_functions() {
    abigen!(
        SimpleContract,
        r#"[
        getValue() (uint256)
        getValue(uint256 otherValue) (uint256)
        getValue(uint256 otherValue, address addr) (uint256)
    ]"#
    );

    let (provider, _) = Provider::mocked();
    let client = Arc::new(provider);
    let contract = SimpleContract::new(Address::zero(), client);
    // ensure both functions are callable
    let _ = contract.get_value();
    let _ = contract.get_value_with_other_value(1337u64.into());
    let _ = contract.get_value_with_other_value_and_addr(1337u64.into(), Address::zero());

    let call = GetValueCall;

    let encoded_call = contract.encode("getValue", ()).unwrap();
    assert_eq!(encoded_call, call.clone().encode().unwrap());
    let decoded_call = GetValueCall::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(call, decoded_call);

    let contract_call = SimpleContractCalls::GetValue(call);
    let decoded_enum = SimpleContractCalls::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(contract_call, decoded_enum);
    assert_eq!(encoded_call, contract_call.encode().unwrap());

    let call = GetValueWithOtherValueCall {
        other_value: 420u64.into(),
    };

    let encoded_call = contract
        .encode_with_selector([15, 244, 201, 22], call.other_value)
        .unwrap();
    assert_eq!(encoded_call, call.clone().encode().unwrap());
    let decoded_call = GetValueWithOtherValueCall::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(call, decoded_call);

    let contract_call = SimpleContractCalls::GetValueWithOtherValue(call);
    let decoded_enum = SimpleContractCalls::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(contract_call, decoded_enum);
    assert_eq!(encoded_call, contract_call.encode().unwrap());

    let call = GetValueWithOtherValueAndAddrCall {
        other_value: 420u64.into(),
        addr: Address::random(),
    };

    let encoded_call = contract
        .encode_with_selector([14, 97, 29, 56], (call.other_value, call.addr))
        .unwrap();
    let decoded_call = GetValueWithOtherValueAndAddrCall::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(call, decoded_call);

    let contract_call = SimpleContractCalls::GetValueWithOtherValueAndAddr(call);
    let decoded_enum = SimpleContractCalls::decode(encoded_call.as_ref()).unwrap();
    assert_eq!(contract_call, decoded_enum);
    assert_eq!(encoded_call, contract_call.encode().unwrap());
}
