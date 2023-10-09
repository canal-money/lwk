mod jade_emulator;
mod sign;
mod test_session;

use bs_containers::testcontainers::clients::Cli;
use jade::protocol::GetXpubParams;
use software_signer::*;
use std::collections::HashSet;
use test_session::*;
use wollet::*;

use crate::{jade_emulator::inner_jade_debug_initialization, sign::Sign};

#[test]
fn liquid() {
    let server = setup();
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let signer = Signer::new(mnemonic, &wollet::EC).unwrap();
    let slip77_key = "9c8e4f05c7711a98c838be228bcb84924d4570ca53f35fa1c793e58841d47023";
    let desc_str = format!("ct(slip77({}),elwpkh({}/*))", slip77_key, signer.xpub());
    let mut wallet = TestElectrumWallet::new(&server.electrs.electrum_url, &desc_str);
    let signers: [Box<dyn Sign>; 1] = [Box::new(signer.clone())];

    let docker = Cli::default();
    let jade_init = inner_jade_debug_initialization(&docker, mnemonic.to_string());
    let signers_with_jade: [Box<dyn Sign>; 2] = [Box::new(signer), Box::new(jade_init.jade)];

    wallet.fund_btc(&server);
    let asset = wallet.fund_asset(&server);

    wallet.send_btc(&signers_with_jade, None);
    let node_address = server.node_getnewaddress();
    wallet.send_asset(&signers, &node_address, &asset, None);
    let node_address1 = server.node_getnewaddress();
    let node_address2 = server.node_getnewaddress();
    wallet.send_many(
        &signers,
        &node_address1,
        &asset,
        &node_address2,
        &wallet.policy_asset(),
        None,
    );
    let (asset, _token) = wallet.issueasset(&signers, 10, 1, "", None);
    wallet.reissueasset(&signers, 10, &asset, None);
    wallet.burnasset(&signers, 5, &asset, None);
}

#[test]
fn view() {
    let server = setup();
    // "view" descriptor
    let xpub = "tpubD6NzVbkrYhZ4Was8nwnZi7eiWUNJq2LFpPSCMQLioUfUtT1e72GkRbmVeRAZc26j5MRUz2hRLsaVHJfs6L7ppNfLUrm9btQTuaEsLrT7D87";
    let descriptor_blinding_key = "L3jXxwef3fpB7hcrFozcWgHeJCPSAFiZ1Ji2YJMPxceaGvy3PC1q";
    let desc_str = format!("ct({},elwpkh({}/*))", descriptor_blinding_key, xpub);
    let mut wallet = TestElectrumWallet::new(&server.electrs.electrum_url, &desc_str);

    wallet.fund_btc(&server);
    let _asset = wallet.fund_asset(&server);

    let descriptor_blinding_key =
        "slip77(9c8e4f05c7711a98c838be228bcb84924d4570ca53f35fa1c793e58841d47023)";
    let desc_str = format!("ct({},elwpkh({}/*))", descriptor_blinding_key, xpub);
    let mut wallet = TestElectrumWallet::new(&server.electrs.electrum_url, &desc_str);

    wallet.fund_btc(&server);
}

#[test]
fn roundtrip() {
    let server = setup();

    let signer1 = generate_signer();
    let slip77_key = generate_slip77();
    let desc1 = format!("ct(slip77({}),elwpkh({}/*))", slip77_key, signer1.xpub());

    let view_key = generate_view_key();
    let signer2 = generate_signer();
    let desc2 = format!("ct({},elwpkh({}/*))", view_key, signer2.xpub());

    let view_key3 = generate_view_key();
    let signer3 = generate_signer();
    let desc3 = format!("ct({},elsh(wpkh({}/*)))", view_key3, signer3.xpub());

    let view_key = generate_view_key();
    let signer4 = generate_signer();
    let desc4 = format!("ct({},elwpkh({}/9/*))", view_key, signer4.xpub());

    let view_key = generate_view_key();
    let signer51 = generate_signer();
    let signer52 = generate_signer();
    let desc5 = format!(
        "ct({},elwsh(multi(2,{}/*,{}/*)))",
        view_key,
        signer51.xpub(),
        signer52.xpub()
    );
    let signers1: [Box<dyn Sign>; 1] = [Box::new(signer1)];
    let signers2: [Box<dyn Sign>; 1] = [Box::new(signer2)];
    let signers3: [Box<dyn Sign>; 1] = [Box::new(signer3)];
    let signers4: [Box<dyn Sign>; 1] = [Box::new(signer4)];
    let signers5: [Box<dyn Sign>; 2] = [Box::new(signer51), Box::new(signer52)];

    // std::thread::scope(|s| {
    for (signers, desc) in [
        (&signers1[..], desc1),
        (&signers2[..], desc2),
        (&signers3[..], desc3),
        (&signers4[..], desc4),
        (&signers5[..], desc5),
    ] {
        let server = &server;
        let mut wallet = TestElectrumWallet::new(&server.electrs.electrum_url, &desc);
        // s.spawn(move || {
        wallet.fund_btc(server);
        server.generate(1);
        wallet.send_btc(signers, None);
        let (asset, _token) = wallet.issueasset(signers, 100_000, 1, "", None);
        let node_address = server.node_getnewaddress();
        wallet.send_asset(signers, &node_address, &asset, None);
        let node_address1 = server.node_getnewaddress();
        let node_address2 = server.node_getnewaddress();
        wallet.send_many(
            signers,
            &node_address1,
            &asset,
            &node_address2,
            &wallet.policy_asset(),
            None,
        );
        wallet.reissueasset(signers, 10_000, &asset, None);
        wallet.burnasset(signers, 5_000, &asset, None);
        server.generate(2);
        // });
    }
    // });
}

#[test]
fn unsupported_descriptor() {
    let signer1 = generate_signer();
    let signer2 = generate_signer();
    let view_key = generate_view_key();
    let desc_p2pkh = format!("ct({},elpkh({}/*))", view_key, signer1.xpub());
    let desc_p2sh = format!(
        "ct({},elsh(multi(2,{}/*,{}/*)))",
        view_key,
        signer1.xpub(),
        signer2.xpub()
    );
    let desc_p2tr = format!("ct({},eltr({}/*))", view_key, signer1.xpub());
    let desc_no_wildcard = format!("ct({},elwpkh({}))", view_key, signer1.xpub());

    for desc in [desc_p2pkh, desc_p2sh, desc_p2tr, desc_no_wildcard] {
        new_unsupported_wallet(&desc, Error::UnsupportedDescriptor);
    }

    let bare_key = "0337cceec0beea0232ebe14cba0197a9fbd45fcf2ec946749de920e71434c2b904";
    let desc_bare = format!("ct({},elwpkh({}/*))", bare_key, signer1.xpub());
    new_unsupported_wallet(&desc_bare, Error::BlindingBareUnsupported);
}

#[test]
fn address() {
    let server = setup();

    let signer = generate_signer();
    let view_key = generate_view_key();
    let desc = format!("ct({},elwpkh({}/*))", view_key, signer.xpub());

    let mut wallet = TestElectrumWallet::new(&server.electrs.electrum_url, &desc);

    let gap_limit: u32 = 20;
    let addresses: Vec<_> = (0..(gap_limit + 1))
        .map(|i| wallet.address_result(Some(i)))
        .collect();

    // First unused address has index 0
    let address = wallet.address_result(None);
    assert_eq!(address.index(), 0);
    for i in 0..(gap_limit + 1) {
        assert_eq!(addresses[i as usize].index(), i);
    }

    // We get all different addresses
    let set: HashSet<_> = addresses.iter().map(|a| a.address()).collect();
    assert_eq!(addresses.len(), set.len());

    let max = addresses.iter().map(|a| a.index()).max().unwrap();
    assert_eq!(max, gap_limit);

    // Fund an address beyond the gap limit
    // Note that we need to find and address before it,
    // otherwise the sync mechanism will not look for those funds
    let satoshi = 10_000;
    let mid_address = addresses[(gap_limit / 2) as usize].clone();
    let last_address = addresses[gap_limit as usize].clone();
    assert_eq!(last_address.index(), gap_limit);
    let mid_address = Some(mid_address.address().clone());
    let last_address = Some(last_address.address().clone());
    wallet.fund(&server, satoshi, mid_address, None);
    wallet.fund(&server, satoshi, last_address, None);
}

#[test]
fn different_blinding_keys() {
    // Two wallet with same "bitcoin" descriptor but different blinding keys
    let server = setup();

    let signer = generate_signer();
    let view_key1 = generate_view_key();
    let view_key2 = generate_view_key();
    let desc1 = format!("ct({},elwpkh({}/*))", view_key1, signer.xpub());
    let desc2 = format!("ct({},elwpkh({}/*))", view_key2, signer.xpub());

    let mut wallet1 = TestElectrumWallet::new(&server.electrs.electrum_url, &desc1);
    wallet1.sync();
    assert_eq!(wallet1.address_result(None).index(), 0);
    wallet1.fund_btc(&server);
    assert_eq!(wallet1.address_result(None).index(), 1);

    let mut wallet2 = TestElectrumWallet::new(&server.electrs.electrum_url, &desc2);
    wallet2.sync();
    assert_eq!(wallet2.address_result(None).index(), 0);
    wallet2.fund_btc(&server);
    assert_eq!(wallet2.address_result(None).index(), 1);
}

#[test]
fn fee_rate() {
    // Use a fee rate different from the default one
    let fee_rate = Some(200.0);

    let server = setup();
    let signer = generate_signer();
    let view_key = generate_view_key();
    let desc = format!("ct({},elwpkh({}/*))", view_key, signer.xpub());
    let signers: [Box<dyn Sign>; 1] = [Box::new(signer)];

    let mut wallet = TestElectrumWallet::new(&server.electrs.electrum_url, &desc);
    wallet.fund_btc(&server);
    wallet.send_btc(&signers, fee_rate);
    let (asset, _token) = wallet.issueasset(&signers, 100_000, 1, "", fee_rate);
    let node_address = server.node_getnewaddress();
    wallet.send_asset(&signers, &node_address, &asset, fee_rate);
    let node_address1 = server.node_getnewaddress();
    let node_address2 = server.node_getnewaddress();
    wallet.send_many(
        &signers,
        &node_address1,
        &asset,
        &node_address2,
        &wallet.policy_asset(),
        fee_rate,
    );
    wallet.reissueasset(&signers, 10_000, &asset, fee_rate);
    wallet.burnasset(&signers, 5_000, &asset, fee_rate);
}

#[test]
fn contract() {
    // Issue an asset with a contract
    let contract = "{\"entity\":{\"domain\":\"test.com\"},\"issuer_pubkey\":\"0337cceec0beea0232ebe14cba0197a9fbd45fcf2ec946749de920e71434c2b904\",\"name\":\"Test\",\"precision\":8,\"ticker\":\"TEST\",\"version\":0}";

    let server = setup();
    let signer = generate_signer();
    let view_key = generate_view_key();
    let desc = format!("ct({},elwpkh({}/*))", view_key, signer.xpub());
    let signers: [Box<dyn Sign>; 1] = [Box::new(signer)];

    let mut wallet = TestElectrumWallet::new(&server.electrs.electrum_url, &desc);
    wallet.fund_btc(&server);
    wallet.send_btc(&signers, None);
    let (_asset, _token) = wallet.issueasset(&signers, 100_000, 1, contract, None);

    // Error cases
    let contract_d = "{\"entity\":{\"domain\":\"testcom\"},\"issuer_pubkey\":\"0337cceec0beea0232ebe14cba0197a9fbd45fcf2ec946749de920e71434c2b904\",\"name\":\"Test\",\"precision\":8,\"ticker\":\"TEST\",\"version\":0}";
    let contract_v = "{\"entity\":{\"domain\":\"test.com\"},\"issuer_pubkey\":\"0337cceec0beea0232ebe14cba0197a9fbd45fcf2ec946749de920e71434c2b904\",\"name\":\"Test\",\"precision\":8,\"ticker\":\"TEST\",\"version\":1}";
    let contract_p = "{\"entity\":{\"domain\":\"test.com\"},\"issuer_pubkey\":\"0337cceec0beea0232ebe14cba0197a9fbd45fcf2ec946749de920e71434c2b904\",\"name\":\"Test\",\"precision\":18,\"ticker\":\"TEST\",\"version\":0}";
    let contract_n = "{\"entity\":{\"domain\":\"test.com\"},\"issuer_pubkey\":\"0337cceec0beea0232ebe14cba0197a9fbd45fcf2ec946749de920e71434c2b904\",\"name\":\"\",\"precision\":8,\"ticker\":\"TEST\",\"version\":0}";
    let contract_t = "{\"entity\":{\"domain\":\"test.com\"},\"issuer_pubkey\":\"0337cceec0beea0232ebe14cba0197a9fbd45fcf2ec946749de920e71434c2b904\",\"name\":\"Test\",\"precision\":8,\"ticker\":\"TT\",\"version\":0}";
    let contract_i = "{\"entity\":{\"domain\":\"test.com\"},\"issuer_pubkey\":\"37cceec0beea0232ebe14cba0197a9fbd45fcf2ec946749de920e71434c2b904\",\"name\":\"Test\",\"precision\":8,\"ticker\":\"TEST\",\"version\":0}";

    for (contract, expected) in [
        (contract_d, Error::InvalidDomain),
        (contract_v, Error::InvalidVersion),
        (contract_p, Error::InvalidPrecision),
        (contract_n, Error::InvalidName),
        (contract_t, Error::InvalidTicker),
        (contract_i, Error::InvalidIssuerPubkey),
    ] {
        let err = wallet
            .electrum_wallet
            .issueasset(10, "", 1, "", contract, None)
            .unwrap_err();
        assert_eq!(err.to_string(), expected.to_string());
    }
}

#[test]
fn multiple_descriptors() {
    // Use a different descriptors for the asset and the reissuance token

    let server = setup();
    // Asset descriptor and signers
    let signer_a = generate_signer();
    let view_key_a = generate_view_key();
    let desc_a = format!("ct({},elwpkh({}/*))", view_key_a, signer_a.xpub());
    let mut wallet_a = TestElectrumWallet::new(&server.electrs.electrum_url, &desc_a);
    // Token descriptor and signers
    let signer_t1 = generate_signer();
    let signer_t2 = generate_signer();
    let view_key_t = generate_view_key();
    let desc_t = format!(
        "ct({},elwsh(multi(2,{}/*,{}/*)))",
        view_key_t,
        signer_t1.xpub(),
        signer_t2.xpub()
    );
    let mut wallet_t = TestElectrumWallet::new(&server.electrs.electrum_url, &desc_t);

    // Fund both wallets
    wallet_a.fund_btc(&server);
    wallet_t.fund_btc(&server);

    // Issue an asset, sending the asset to asset wallet, and the token to the token wallet
    let satoshi_a = 100_000;
    let satoshi_t = 1;
    let address_t = wallet_t.address().to_string();
    let mut pset = wallet_a
        .electrum_wallet
        .issueasset(satoshi_a, "", satoshi_t, &address_t, "", None)
        .unwrap();
    let (asset, token) = &pset.inputs()[0].issuance_ids();
    let details_a = wallet_a.electrum_wallet.get_details(&pset).unwrap();
    let details_t = wallet_t.electrum_wallet.get_details(&pset).unwrap();
    assert_eq!(*details_a.balances.get(asset).unwrap(), satoshi_a as i64);
    // FIXME: get_details needs bip32 derivation to be set
    //assert_eq!(*details_t.balances.get(token).unwrap(), satoshi_t  as i64);
    assert!(details_t.balances.get(token).is_none());
    wallet_a.sign(&signer_a, &mut pset);
    wallet_a.send(&mut pset);
    assert_eq!(wallet_a.balance(asset), satoshi_a);
    assert_eq!(wallet_t.balance(token), satoshi_t);

    // Reissue the asset, sending the asset to asset wallet, and keeping the token in the token
    // wallet
    let satoshi_ar = 1_000;
    let address_a = wallet_a.address().to_string();
    let mut pset = wallet_t
        .electrum_wallet
        .reissueasset(asset.to_string().as_str(), satoshi_ar, &address_a, None)
        .unwrap();
    let details_a = wallet_a.electrum_wallet.get_details(&pset).unwrap();
    let details_t = wallet_t.electrum_wallet.get_details(&pset).unwrap();
    // FIXME: get_details needs bip32 derivation to be set
    //assert_eq!(*details_a.balances.get(asset).unwrap(), satoshi_ar as i64);
    assert!(details_a.balances.get(asset).is_none());
    assert_eq!(*details_t.balances.get(token).unwrap(), 0i64);
    wallet_t.sign(&signer_t1, &mut pset);
    wallet_t.sign(&signer_t2, &mut pset);
    wallet_t.send(&mut pset);
    assert_eq!(wallet_a.balance(asset), satoshi_a + satoshi_ar);
    assert_eq!(wallet_t.balance(token), satoshi_t);
}

#[test]
fn createpset_error() {
    let server = setup();
    let signer = generate_signer();
    let view_key = generate_view_key();
    let desc = format!("ct({},elwpkh({}/*))", view_key, signer.xpub());

    let mut wallet = TestElectrumWallet::new(&server.electrs.electrum_url, &desc);
    wallet.fund_btc(&server);
    let satoshi_a = 100_000;
    let satoshi_t = 1;
    let (asset, token) =
        wallet.issueasset(&[Box::new(signer.clone())], satoshi_a, satoshi_t, "", None);
    let asset = asset.to_string();
    let token = token.to_string();

    // Invalid address
    let addressees = vec![UnvalidatedAddressee {
        satoshi: 1_000,
        address: "",
        asset: "",
    }];
    let err = wallet
        .electrum_wallet
        .sendmany(addressees, None)
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "base58 error: base58ck data not even long enough for a checksum".to_string()
    );

    // Not confidential address
    let mut address = wallet.address();
    address.blinding_pubkey = None;
    let not_conf_address = address.to_string();
    let addressees = vec![UnvalidatedAddressee {
        satoshi: 1_000,
        address: &not_conf_address,
        asset: "",
    }];
    let err = wallet
        .electrum_wallet
        .sendmany(addressees, None)
        .unwrap_err();
    assert_eq!(err.to_string(), Error::NotConfidentialAddress.to_string());

    let address = wallet.address().to_string();
    // Invalid amount
    let addressees = vec![UnvalidatedAddressee {
        satoshi: 0,
        address: &address,
        asset: "",
    }];
    let err = wallet
        .electrum_wallet
        .sendmany(addressees, None)
        .unwrap_err();
    assert_eq!(err.to_string(), Error::InvalidAmount.to_string());

    // Invalid asset
    let addressees = vec![UnvalidatedAddressee {
        satoshi: 1_000,
        address: &address,
        asset: "aaaa",
    }];
    let err = wallet
        .electrum_wallet
        .sendmany(addressees, None)
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "bad hex string length 4 (expected 64)".to_string()
    );

    // Insufficient funds
    // Not enough lbtc
    let addressees = vec![UnvalidatedAddressee {
        satoshi: 2_200_000_000_000_000,
        address: &address,
        asset: "",
    }];
    let err = wallet
        .electrum_wallet
        .sendmany(addressees, None)
        .unwrap_err();
    assert_eq!(err.to_string(), Error::InsufficientFunds.to_string());

    // Not enough asset
    let addressees = vec![UnvalidatedAddressee {
        satoshi: satoshi_a + 1,
        address: &address,
        asset: &asset,
    }];
    let err = wallet
        .electrum_wallet
        .sendmany(addressees, None)
        .unwrap_err();
    assert_eq!(err.to_string(), Error::InsufficientFunds.to_string());

    // Not enough token
    let signer2 = generate_signer();
    let view_key2 = generate_view_key();
    let desc2 = format!("ct({},elwpkh({}/*))", view_key2, signer2.xpub());
    let wallet2 = TestElectrumWallet::new(&server.electrs.electrum_url, &desc2);

    // Send token elsewhere
    let address = wallet2.address();
    let mut pset = wallet
        .electrum_wallet
        .sendasset(satoshi_t, &address.to_string(), &token, None)
        .unwrap();
    wallet.sign(&signer, &mut pset);
    wallet.send(&mut pset);

    let err = wallet
        .electrum_wallet
        .reissueasset(&asset, satoshi_a, "", None)
        .unwrap_err();
    assert_eq!(err.to_string(), Error::InsufficientFunds.to_string());

    // The other wallet is unaware of the issuance transaction,
    // so it can't reissue the asset.
    let err = wallet2
        .electrum_wallet
        .reissueasset(&asset, satoshi_a, "", None)
        .unwrap_err();
    assert_eq!(err.to_string(), Error::MissingIssuance.to_string());
}

#[test]
fn jade_sign_wollet_pset() {
    let server = setup();
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let signer = Signer::new(mnemonic, &wollet::EC).unwrap();
    let slip77_key = "9c8e4f05c7711a98c838be228bcb84924d4570ca53f35fa1c793e58841d47023";
    let desc_str = format!("ct(slip77({}),elwpkh({}/*))", slip77_key, signer.xpub());
    let mut wallet = TestElectrumWallet::new(&server.electrs.electrum_url, &desc_str);

    wallet.fund_btc(&server);

    let my_addr = wallet.address();

    let mut pset = wallet
        .electrum_wallet
        .sendlbtc(1000, &my_addr.to_string(), None)
        .unwrap();

    let docker = Cli::default();
    let jade_init = inner_jade_debug_initialization(&docker, mnemonic.to_string());

    let jade_xpub = jade_init
        .jade
        .get_xpub(GetXpubParams {
            network: jade::Network::LocaltestLiquid,
            path: vec![],
        })
        .unwrap();
    assert_eq!(jade_xpub.get(), signer.xpub().to_string());

    let signatures_added = jade_init.jade.sign(&mut pset).unwrap();
    assert_eq!(signatures_added, 1);

    wallet.send(&mut pset);
}