use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokenizers::Tokenizer;
use tract_onnx::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct AIAnalysis {
    pub fact_check_score: f32,
    pub source_reliability: f32,
    pub content_quality: f32,
    pub bias_detection: f32,
    pub detected_entities: Vec<String>,
    pub confidence_scores: std::collections::HashMap<String, f32>,
}

pub struct AIModel {
    model: SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>,
    tokenizer: Tokenizer,
    device: Device,
}

impl AIModel {
    pub fn new(model_path: &str) -> Result<Self> {
        let model_path = Path::new(model_path);
        if !model_path.exists() {
            return Err(AppError::Internal(format!(
                "Model not found at path: {}",
                model_path.display()
            )));
        }

        // Load the ONNX model
        let model = tract_onnx::onnx()
            .model_for_path(model_path)?
            .into_optimized()?
            .into_runnable()?;

        // Load the tokenizer
        let tokenizer = Tokenizer::from_file(model_path.with_extension("tokenizer.json"))
            .map_err(|e| AppError::Internal(format!("Failed to load tokenizer: {}", e)))?;

        Ok(Self {
            model,
            tokenizer,
            device: Device::Cpu,
        })
    }

    pub async fn analyze_content(&self, content: &str) -> Result<AIAnalysis> {
        // Tokenize the input
        let tokens = self.tokenizer
            .encode(content, true)
            .map_err(|e| AppError::Internal(format!("Tokenization failed: {}", e)))?;

        // Prepare input tensor
        let input = tract_ndarray::Array4::from_shape_fn((1, 1, tokens.len(), 1), |(_, _, i, _)| {
            tokens.get_ids()[i] as f32
        });

        // Run inference
        let result = self.model.run(tvec!(input.into()))?;
        let output = result[0].to_array_view::<f32>()?;

        // Process the model output
        let fact_check_score = output[[0, 0, 0, 0]];
        let source_reliability = output[[0, 0, 1, 0]];
        let content_quality = output[[0, 0, 2, 0]];
        let bias_detection = output[[0, 0, 3, 0]];

        // Extract entities using NER model
        let entities = self.extract_entities(content)?;

        Ok(AIAnalysis {
            fact_check_score: fact_check_score.clamp(0.0, 1.0),
            source_reliability: source_reliability.clamp(0.0, 1.0),
            content_quality: content_quality.clamp(0.0, 1.0),
            bias_detection: bias_detection.clamp(0.0, 1.0),
            detected_entities: entities,
            confidence_scores: self.calculate_confidence_scores(&output)?,
        })
    }

    fn extract_entities(&self, content: &str) -> Result<Vec<String>> {
        // TODO: Implement NER model inference
        // For now, return a simple implementation
        Ok(vec!["person".to_string(), "organization".to_string()])
    }

    fn calculate_confidence_scores(
        &self,
        output: &tract_ndarray::ArrayView4<f32>,
    ) -> Result<std::collections::HashMap<String, f32>> {
        let mut scores = std::collections::HashMap::new();
        scores.insert("fact_check".to_string(), output[[0, 0, 0, 0]].clamp(0.0, 1.0));
        scores.insert("source_reliability".to_string(), output[[0, 0, 1, 0]].clamp(0.0, 1.0));
        scores.insert("content_quality".to_string(), output[[0, 0, 2, 0]].clamp(0.0, 1.0));
        scores.insert("bias_detection".to_string(), output[[0, 0, 3, 0]].clamp(0.0, 1.0));
        Ok(scores)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_model() {
        let model = AIModel::new("models/news_verification.onnx").unwrap();
        let content = "This is a test article about a new technology breakthrough.";
        let analysis = model.analyze_content(content).await.unwrap();

        assert!(analysis.fact_check_score >= 0.0 && analysis.fact_check_score <= 1.0);
        assert!(analysis.source_reliability >= 0.0 && analysis.source_reliability <= 1.0);
        assert!(analysis.content_quality >= 0.0 && analysis.content_quality <= 1.0);
        assert!(analysis.bias_detection >= 0.0 && analysis.bias_detection <= 1.0);
    }
} 