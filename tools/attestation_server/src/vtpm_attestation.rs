use tss_esapi::{
    abstraction::{
        ak::{create_ak, load_ak},
        ek::{create_ek_public_from_default_template, retrieve_ek_pubcert},
        AsymmetricAlgorithmSelection
    },
    attributes::{ObjectAttributesBuilder, SessionAttributesBuilder},
    constants::SessionType,
    handles::{AuthHandle, KeyHandle, SessionHandle},
    interface_types::{
        algorithm::{HashingAlgorithm, PublicAlgorithm, SignatureSchemeAlgorithm},
        ecc::EccCurve,
        key_bits::RsaKeyBits,
        resource_handles::Hierarchy,
        session_handles::PolicySession,
        algorithm::AsymmetricAlgorithm
    },
    structures::{
        Data, Digest, EccPoint, EccScheme, HashScheme, MaxBuffer, PublicBuilder,
        PublicEccParametersBuilder, SignatureScheme, SymmetricCipherParameters,
        SymmetricDefinition, SymmetricDefinitionObject,
        PcrSelectionListBuilder, PcrSlot, Attest, Signature, Public
    },
    traits::{Marshall, UnMarshall},
    Context, TctiNameConf,
};
use std::convert::{TryFrom, TryInto};
use std::env;
use base64;

use openssl::{
    bn::BigNum,
    pkey::PKey,
    rsa::{Padding, Rsa},
    sign::Verifier,
    hash::MessageDigest,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct VTPMQuote {
    attest: String,
    signature: String,
    ak_pub: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the TPM via the socket interface
    unsafe {
        env::set_var("TPM2TOOLS_TCTI", "swtpm:port=2321");
    }

    let nonce: u64 = 1234567890; // Example nonce, replace with actual nonce generation logic
    let (attest, signature, ak_pub) = get_quote(nonce)?;

    let att_marsh = attest.marshall().expect("Failed to marshal attest");
    let sig_marsh = signature.marshall().expect("Failed to marshal signature");  
    let ak_pub_marsh = ak_pub.marshall().expect("Failed to marshal AK public key");

    let att_b64 = base64::encode(att_marsh);
    println!("digest b64: {}", att_b64);
    let sig_b64 = base64::encode(sig_marsh);
    println!("sig b64: {}", sig_b64);
    let ak_pub_b64 = base64::encode(ak_pub_marsh);
    println!("ak pubkey b64: {}", ak_pub_b64);

    validate_quote(&att_b64, &sig_b64, &ak_pub_b64, &nonce.to_be_bytes().to_vec())?;

    Ok(())
}

pub fn validate_quote(
    attest_b64: &str,
    signature_b64: &str,
    ak_pub_b64: &str,
    nonce: &Vec<u8>
) -> Result<(), Box<dyn std::error::Error>> {

    let attest_bytes = base64::decode(attest_b64)?;
    let signature_bytes = base64::decode(signature_b64)?;
    let ak_pub_bytes = base64::decode(ak_pub_b64)?;

    let attest: Attest = Attest::unmarshall(&attest_bytes)?;
    let signature: Signature = Signature::unmarshall(&signature_bytes)?;
    let ak_pub: Public = Public::unmarshall(&ak_pub_bytes)?;

    //println!("Attestation: {:?}", attest);
    //println!("Signature: {:?}", signature);
    //println!("AK Public: {:?}", ak_pub);

    // Check nonce in quote matches expected nonce
    let extra_data = attest.extra_data();
    if extra_data.value() != nonce {
        return Err("Nonce in quote does not match expected nonce".into());
    }

    // Extract modulus bytes from the RSA public key unique field
    let modulus_bytes = match &ak_pub {
        Public::Rsa { unique, .. } => unique.value(),
        _ => return Err("Public key is not RSA".into()),
    };

    // Exponent is 65537 unless specified otherwise
    let exponent = 65537u32;

    // Extract signature bytes from Signature
    let signature_bytes_a = match &signature {
        Signature::RsaPss(rsa_sig) => rsa_sig.signature().value(),
        _ => return Err("Signature is not RsaPss".into()),
    };

    // Construct RSA public key from modulus and exponent
    let n = BigNum::from_slice(modulus_bytes)?;
    let e = BigNum::from_u32(exponent)?;

    let rsa_pub = Rsa::from_public_components(n, e)?;
    let public_key = PKey::from_rsa(rsa_pub)?;

    // Create verifier with RSA-PSS and SHA256
    let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key)?;
    verifier.set_rsa_padding(Padding::PKCS1_PSS)?;
    verifier.set_rsa_pss_saltlen(openssl::sign::RsaPssSaltlen::DIGEST_LENGTH)?;

    // Verify signature over the attestation bytes (raw marshalled bytes)
    verifier.update(&attest_bytes)?;
    let is_valid = verifier.verify(signature_bytes_a)?;

    if !is_valid {
        return Err("Signature verification failed".into());
    }

    println!("Signature is valid");
    println!("Quote verified successfully!");

    Ok(())
}

pub fn get_quote(nonce: u64) -> Result<(Attest, Signature, Public), Box<dyn std::error::Error>> {
    let mut context = Context::new(
        TctiNameConf::from_environment_variable()
            .expect("Failed to get TCTI / TPM2TOOLS_TCTI from environment. Try `export TCTI=device:/dev/tpmrm0`"),
    )
    .expect("Failed to create Context");


    let ek_alg = AsymmetricAlgorithmSelection::Rsa(RsaKeyBits::Rsa2048);
    let hash_alg = HashingAlgorithm::Sha256;
    let sign_alg = SignatureSchemeAlgorithm::RsaPss;
    let sig_scheme = SignatureScheme::RsaPss {
        hash_scheme: HashScheme::new(hash_alg),
    };

    // Create EK (Endorsement Key) public template
    let ek_template = create_ek_public_from_default_template(AsymmetricAlgorithm::Rsa, None).unwrap();

    let ek_handle = context
        .execute_with_nullauth_session(|ctx| {
            ctx.create_primary(Hierarchy::Endorsement, ek_template, None, None, None, None)
        })
        .expect("Failed to load ek_template")
        .key_handle;

    let ak_create_result = create_ak(
        &mut context,
        ek_handle,
        hash_alg,
        sign_alg,
        None,
        None,
    )
    .expect("Failed to create attestation key");

    let ak_public = ak_create_result.out_public.clone();

    // For later, we'll load the AIK now and save it's context.
    let ak_handle = load_ak(
        &mut context,
        ek_handle,
        None,
        ak_create_result.out_private,
        ak_create_result.out_public,
    )
    .expect("Failed to load attestation key");

    // let ak_context = context
    //     .execute_with_nullauth_session(|ctx| ctx.context_save(ak_handle.into()))
    //     .expect("Failed to save ak context");

    // context
    //     .flush_context(ak_handle.into())
    //     .expect("Unable to flush ak_handle");

    // // For now to save resources, we save the ek context.
    // let ek_context = context
    //     .execute_with_nullauth_session(|ctx| ctx.context_save(ek_handle.into()))
    //     .expect("Failed to save ek context");

    // Get EK (Endorsement Key) handle


    let auth_session = context.start_auth_session(
        None,
        None,
        None,
        SessionType::Hmac,
        SymmetricDefinition::AES_256_CFB,
        HashingAlgorithm::Sha256
    )
    .expect("Failed to create session")
    .expect("Received invalid handle");

    context.set_sessions((Some(auth_session), None, None));

    //Generate nonce
    let nonce = Data::try_from(nonce.to_be_bytes().to_vec())?;

    // Define the PCR selection list
    let pcr_selection_list = PcrSelectionListBuilder::new()
    .with_selection(HashingAlgorithm::Sha256, &[PcrSlot::Slot0, PcrSlot::Slot1])
    .build()
    .expect("Failed to build PcrSelectionList");

    let (attest, signature) = context.quote(
        ak_handle,
        nonce,
        sig_scheme,
        pcr_selection_list
    )?;

    //println!("Attestation: {:?}", attest);
    //println!("Signature: {:?}", signature);
    //println!("AK Public: {:?}", ak_public);

    Ok((attest, signature, ak_public))
}

pub fn get_vtpm_quote(nonce: u64) -> Result<VTPMQuote, Box<dyn std::error::Error>> {
    let (attest, signature, ak_pub) = get_quote(nonce)?;

    let att_marsh = attest.marshall().expect("Failed to marshal attest");
    let sig_marsh = signature.marshall().expect("Failed to marshal signature");  
    let ak_pub_marsh = ak_pub.marshall().expect("Failed to marshal AK public key");

    let att_b64 = base64::encode(att_marsh);
    //println!("digest b64: {}", att_b64);
    let sig_b64 = base64::encode(sig_marsh);
    //println!("sig b64: {}", sig_b64);
    let ak_pub_b64 = base64::encode(ak_pub_marsh);
    //println!("ak pubkey b64: {}", ak_pub_b64);

    Ok(VTPMQuote {
        attest: att_b64,
        signature: sig_b64,
        ak_pub: ak_pub_b64,
    })
}