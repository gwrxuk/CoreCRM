use crate::config::{DatabaseConfig, KafkaConfig, RedisConfig};
use rdkafka::config::ClientConfig;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

pub async fn init_pool(config: &DatabaseConfig) -> Result<sqlx::PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect_timeout(Duration::from_secs(config.connect_timeout))
        .connect(&config.url)
        .await
}

pub async fn init_redis(config: &RedisConfig) -> Result<redis::Client, redis::RedisError> {
    let client = redis::Client::open(config.url.as_str())?;
    Ok(client)
}

pub fn init_kafka_producer(config: &KafkaConfig) -> Result<rdkafka::producer::FutureProducer, rdkafka::error::KafkaError> {
    let producer: rdkafka::producer::FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", config.brokers.join(","))
        .set("client.id", &config.client_id)
        .set("group.id", &config.group_id)
        .set("message.timeout.ms", "5000")
        .create()?;

    Ok(producer)
}

pub async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        let config = DatabaseConfig {
            url: "postgres://postgres:postgres@localhost:5432/core_crm_test".to_string(),
            max_connections: 5,
            min_connections: 1,
            connect_timeout: 5,
        };

        let pool = init_pool(&config).await;
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_redis_connection() {
        let config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            pool_size: 5,
        };

        let client = init_redis(&config).await;
        assert!(client.is_ok());
    }
} 