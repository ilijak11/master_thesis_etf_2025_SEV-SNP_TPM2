use crate::vtpm_attestation::VTPMQuote;
use sev::firmware::guest::AttestationReport;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AttestationReportResponse {
    pub snp_att_report: AttestationReport,
    pub vtpm_quote: VTPMQuote,
}