use crate::{
    core::news_verification::{NewsArticle, NewsVerificationService, VerificationResult},
    error::Result,
};
use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

pub async fn verify_article(
    State(service): State<NewsVerificationService>,
    Json(article): Json<NewsArticle>,
) -> Result<Json<VerificationResult>> {
    let result = service.verify_article(&article).await?;
    Ok(Json(result))
}

pub async fn get_verification_status(
    State(service): State<NewsVerificationService>,
    Path(article_id): Path<Uuid>,
) -> Result<Json<VerificationResult>> {
    // TODO: Implement fetching verification status from database
    // This is a placeholder implementation
    Ok(Json(VerificationResult {
        article_id,
        credibility_score: 0.85,
        ai_analysis: crate::core::news_verification::AIAnalysis {
            fact_check_score: 0.85,
            source_reliability: 0.9,
            content_quality: 0.8,
            bias_detection: 0.1,
            detected_entities: vec!["person".to_string(), "organization".to_string()],
            confidence_scores: std::collections::HashMap::new(),
        },
        blockchain_proof: crate::core::news_verification::BlockchainProof {
            transaction_hash: "0x123...".to_string(),
            block_number: 12345,
            timestamp: chrono::Utc::now(),
            smart_contract_state: "verified".to_string(),
        },
        verification_timestamp: chrono::Utc::now(),
    }))
}

pub async fn get_blockchain_proof(
    State(service): State<NewsVerificationService>,
    Path(article_id): Path<Uuid>,
) -> Result<Json<crate::core::news_verification::BlockchainProof>> {
    // TODO: Implement fetching blockchain proof from database
    // This is a placeholder implementation
    Ok(Json(crate::core::news_verification::BlockchainProof {
        transaction_hash: "0x123...".to_string(),
        block_number: 12345,
        timestamp: chrono::Utc::now(),
        smart_contract_state: "verified".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::news_verification::{AIModel, BlockchainClient};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_verify_article_endpoint() {
        let ai_model = AIModel::new("models/news_verification".to_string());
        let blockchain_client = BlockchainClient::new(
            "https://mainnet.infura.io/v3/your-project-id".to_string(),
            "0x123...".to_string(),
        );
        let service = NewsVerificationService::new(ai_model, blockchain_client);

        let app = Router::new()
            .route("/api/v1/news/verify", axum::routing::post(verify_article))
            .with_state(service);

        let article = NewsArticle {
            id: Uuid::new_v4(),
            title: "Test Article".to_string(),
            content: "This is a test article content.".to_string(),
            source_url: "https://example.com/article".to_string(),
            author: "Test Author".to_string(),
            published_at: chrono::Utc::now(),
            verification_status: crate::core::news_verification::VerificationStatus::Pending,
            credibility_score: 0.0,
            blockchain_hash: "".to_string(),
            smart_contract_address: "".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/news/verify")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&article).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
} 