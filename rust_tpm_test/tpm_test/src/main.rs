use tss_esapi::{
    Context,
    TctiNameConf,
    interface_types::algorithm::HashingAlgorithm,
    structures::{PcrSelectionListBuilder, PcrSlot}
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

    
    let pcr_selection_list = PcrSelectionListBuilder::new()
    .with_selection(HashingAlgorithm::Sha256, &[PcrSlot::Slot0, PcrSlot::Slot1])
    .build()
    .expect("Failed to build PcrSelectionList");

    let (update_counter, read_pcr_list, digest_list) = context.pcr_read(pcr_selection_list)
    .expect("Call to pcr_read failed");

    println!("PCR Slot: {:?}", read_pcr_list);


    Ok(())
}