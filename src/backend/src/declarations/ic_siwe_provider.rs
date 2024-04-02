// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Deserialize, Principal, Encode, Decode};
use ic_cdk::api::call::CallResult as Result;

#[derive(Debug, CandidType, Deserialize)]
pub struct SettingsInput {
  pub uri: String,
  pub domain: String,
  pub statement: Option<String>,
  pub scheme: Option<String>,
  pub salt: String,
  pub session_expires_in: Option<u64>,
  pub targets: Option<Vec<String>>,
  pub chain_id: Option<candid::Nat>,
  pub sign_in_expires_in: Option<u64>,
}

pub type Principal_ = serde_bytes::ByteBuf;
pub type Address = String;
#[derive(Debug, CandidType, Deserialize)]
pub enum GetAddressResponse { Ok(Address), Err(String) }

#[derive(Debug, CandidType, Deserialize)]
pub enum GetPrincipalResponse { Ok(Principal_), Err(String) }

pub type PublicKey = serde_bytes::ByteBuf;
pub type SessionKey = PublicKey;
pub type Timestamp = u64;
#[derive(Debug, CandidType, Deserialize)]
pub struct Delegation {
  pub pubkey: PublicKey,
  pub targets: Option<Vec<Principal>>,
  pub expiration: Timestamp,
}

#[derive(Debug, CandidType, Deserialize)]
pub struct SignedDelegation {
  pub signature: serde_bytes::ByteBuf,
  pub delegation: Delegation,
}

#[derive(Debug, CandidType, Deserialize)]
pub enum GetDelegationResponse { Ok(SignedDelegation), Err(String) }

pub type SiweSignature = String;
pub type CanisterPublicKey = PublicKey;
#[derive(Debug, CandidType, Deserialize)]
pub struct LoginDetails {
  pub user_canister_pubkey: CanisterPublicKey,
  pub expiration: Timestamp,
}

#[derive(Debug, CandidType, Deserialize)]
pub enum LoginResponse { Ok(LoginDetails), Err(String) }

pub type SiweMessage = String;
#[derive(Debug, CandidType, Deserialize)]
pub enum PrepareLoginResponse { Ok(SiweMessage), Err(String) }

pub struct IcSiweProvider(pub Principal);
impl IcSiweProvider {
  pub async fn get_address(&self, arg0: Principal_) -> Result<
    (GetAddressResponse,)
  > { ic_cdk::call(self.0, "get_address", (arg0,)).await }
  pub async fn get_caller_address(&self) -> Result<(GetAddressResponse,)> {
    ic_cdk::call(self.0, "get_caller_address", ()).await
  }
  pub async fn get_principal(&self, arg0: Address) -> Result<
    (GetPrincipalResponse,)
  > { ic_cdk::call(self.0, "get_principal", (arg0,)).await }
  pub async fn siwe_get_delegation(
    &self,
    arg0: Address,
    arg1: SessionKey,
    arg2: Timestamp,
  ) -> Result<(GetDelegationResponse,)> {
    ic_cdk::call(self.0, "siwe_get_delegation", (arg0,arg1,arg2,)).await
  }
  pub async fn siwe_login(
    &self,
    arg0: SiweSignature,
    arg1: Address,
    arg2: SessionKey,
  ) -> Result<(LoginResponse,)> {
    ic_cdk::call(self.0, "siwe_login", (arg0,arg1,arg2,)).await
  }
  pub async fn siwe_prepare_login(&self, arg0: Address) -> Result<
    (PrepareLoginResponse,)
  > { ic_cdk::call(self.0, "siwe_prepare_login", (arg0,)).await }
}
pub const CANISTER_ID : Principal = Principal::from_slice(&[0, 0, 0, 0, 1, 96, 71, 66, 1, 1]); // vis7i-yyaaa-aaaal-ai5ba-cai
pub const ic_siwe_provider : IcSiweProvider = IcSiweProvider(CANISTER_ID);