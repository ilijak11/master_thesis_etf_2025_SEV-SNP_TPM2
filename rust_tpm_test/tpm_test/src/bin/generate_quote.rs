use tss_esapi::{
    Context,
    TctiNameConf,
    interface_types::algorithm::{HashingAlgorithm, AsymmetricAlgorithm, SignatureSchemeAlgorithm},
    structures::{PcrSelectionListBuilder, PcrSlot, Data, SignatureScheme, HashScheme, SymmetricDefinition},
    abstraction::{ek, ak},
    constants::session_type::SessionType,
};

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the TPM via the socket interface
    unsafe {
        env::set_var("TPM2TOOLS_TCTI", "swtpm:port=2321");
    }
    
    
    let mut context = Context::new(
        TctiNameConf::from_environment_variable()
            .expect("Failed to get TCTI / TPM2TOOLS_TCTI from environment. Try `export TCTI=device:/dev/tpmrm0`"),
    )
    .expect("Failed to create Context");

    // Get EK (Endorsement Key) handle
    let ek_handle = ek::create_ek_object(&mut context, AsymmetricAlgorithm::Rsa, None)
        .expect("Failed to create EK");


    //Create AK (Attestation Key)
    let ak_creation_result = ak::create_ak(
        &mut context, 
        ek_handle, 
        HashingAlgorithm::Sha256, 
        SignatureSchemeAlgorithm::RsaSsa, 
        None, 
        None
    )
        .expect("Failed to create AK");

    println!("AK Public: {:?}", ak_creation_result.out_public);
    println!("AK Private: {:?}", ak_creation_result.out_private);

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

    let ak_handle = ak::load_ak(
        &mut context, 
        ek_handle, 
        None, 
        ak_creation_result.out_private, 
        ak_creation_result.out_public
    )
        .expect("Failed to load AK");

    //Generate nonce
    let nonce = Data::try_from(vec![0u8; 16])?;

    println!("{:?}", nonce);

    // Define the PCR selection list
    let pcr_selection_list = PcrSelectionListBuilder::new()
    .with_selection(HashingAlgorithm::Sha256, &[PcrSlot::Slot0, PcrSlot::Slot1])
    .build()
    .expect("Failed to build PcrSelectionList");

    // let (attest, signature) = context.quote(
    //     ak_handle,
    //     nonce,
    //     SignatureScheme::RsaSsa(HashScheme::new(HashingAlgorithm::Sha256)),
    //     pcr_selection,
    // )?;

    Ok(())
}