use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use web3::{
    contract::{Contract, Options},
    types::{Address, H256, U256},
    Web3,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainProof {
    pub transaction_hash: String,
    pub block_number: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub smart_contract_state: String,
}

pub struct BlockchainClient {
    web3: Web3<web3::transports::Http>,
    contract: Contract<web3::transports::Http>,
    contract_address: Address,
}

impl BlockchainClient {
    pub fn new(network_url: &str, contract_address: &str) -> Result<Self> {
        let transport = web3::transports::Http::new(network_url)
            .map_err(|e| AppError::Internal(format!("Failed to create Web3 transport: {}", e)))?;
        let web3 = Web3::new(transport);

        let contract_address = Address::from_str(contract_address)
            .map_err(|e| AppError::Internal(format!("Invalid contract address: {}", e)))?;

        // Load the contract ABI
        let contract_abi = include_bytes!("../../contracts/NewsVerification.json");
        let contract = Contract::from_json(
            web3.eth(),
            contract_address,
            contract_abi,
        ).map_err(|e| AppError::Internal(format!("Failed to load contract: {}", e)))?;

        Ok(Self {
            web3,
            contract,
            contract_address,
        })
    }

    pub async fn create_verification_proof(
        &self,
        article_hash: &str,
        verification_data: &str,
    ) -> Result<BlockchainProof> {
        // Convert article hash to bytes32
        let article_hash_bytes = H256::from_str(article_hash)
            .map_err(|e| AppError::Internal(format!("Invalid article hash: {}", e)))?;

        // Call the smart contract to create verification proof
        let result = self.contract.call(
            "createVerificationProof",
            (article_hash_bytes, verification_data),
            None,
            Options::default(),
            None,
        ).await.map_err(|e| AppError::Internal(format!("Failed to create verification proof: {}", e)))?;

        // Get transaction receipt
        let receipt = self.web3.eth()
            .transaction_receipt(result)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get transaction receipt: {}", e)))?
            .ok_or_else(|| AppError::Internal("Transaction receipt not found".to_string()))?;

        // Get block information
        let block = self.web3.eth()
            .block(receipt.block_number.unwrap())
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get block: {}", e)))?
            .ok_or_else(|| AppError::Internal("Block not found".to_string()))?;

        // Get contract state
        let state: String = self.contract.query(
            "getVerificationState",
            (article_hash_bytes,),
            None,
            Options::default(),
            None,
        ).await.map_err(|e| AppError::Internal(format!("Failed to get verification state: {}", e)))?;

        Ok(BlockchainProof {
            transaction_hash: format!("0x{:x}", receipt.transaction_hash),
            block_number: receipt.block_number.unwrap().as_u64(),
            timestamp: chrono::DateTime::from_timestamp(block.timestamp.as_u64() as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now()),
            smart_contract_state: state,
        })
    }

    pub async fn verify_proof(&self, article_hash: &str) -> Result<bool> {
        let article_hash_bytes = H256::from_str(article_hash)
            .map_err(|e| AppError::Internal(format!("Invalid article hash: {}", e)))?;

        let is_verified: bool = self.contract.query(
            "verifyProof",
            (article_hash_bytes,),
            None,
            Options::default(),
            None,
        ).await.map_err(|e| AppError::Internal(format!("Failed to verify proof: {}", e)))?;

        Ok(is_verified)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blockchain_client() {
        let client = BlockchainClient::new(
            "http://localhost:8545",
            "0x1234567890123456789012345678901234567890",
        ).unwrap();

        let article_hash = "0x1234567890123456789012345678901234567890123456789012345678901234";
        let verification_data = "{\"score\": 0.95, \"timestamp\": 1234567890}";

        let proof = client.create_verification_proof(article_hash, verification_data).await.unwrap();
        assert!(!proof.transaction_hash.is_empty());
        assert!(proof.block_number > 0);

        let is_verified = client.verify_proof(article_hash).await.unwrap();
        assert!(is_verified);
    }
} 