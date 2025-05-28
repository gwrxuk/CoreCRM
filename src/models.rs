use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub source: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEventRequest {
    pub event_type: String,
    pub source: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsQuery {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub event_types: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsResponse {
    pub total_events: i64,
    pub events_by_type: std::collections::HashMap<String, i64>,
    pub events_by_source: std::collections::HashMap<String, i64>,
    pub time_series: Vec<TimeSeriesData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeriesData {
    pub timestamp: DateTime<Utc>,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = Event {
            id: Uuid::new_v4(),
            event_type: "page_view".to_string(),
            source: "web".to_string(),
            data: serde_json::json!({
                "page": "/home",
                "user_id": "123"
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event.event_type, deserialized.event_type);
        assert_eq!(event.source, deserialized.source);
    }

    #[test]
    fn test_analytics_query_validation() {
        let query = AnalyticsQuery {
            start_date: Utc::now(),
            end_date: Utc::now(),
            event_types: Some(vec!["page_view".to_string()]),
            sources: Some(vec!["web".to_string()]),
        };

        let serialized = serde_json::to_string(&query).unwrap();
        let deserialized: AnalyticsQuery = serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            query.event_types.unwrap()[0],
            deserialized.event_types.unwrap()[0]
        );
    }
} 