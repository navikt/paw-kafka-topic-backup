use std::error::Error;

use rdkafka::consumer::{Consumer, StreamConsumer};
use sqlx::PgPool;

use crate::kafka::{config::ApplicationKafkaConfig, hwm::HwmRebalanceHandler};

pub fn create_kafka_consumer(
    pg_pool: PgPool,
    app_config: ApplicationKafkaConfig,
    topics: &[&str],
) -> Result<StreamConsumer<HwmRebalanceHandler>, Box<dyn Error>> {
    let config = app_config.rdkafka_config()?;
    let context = HwmRebalanceHandler { pg_pool };
    let consumer: StreamConsumer<HwmRebalanceHandler> = config.create_with_context(context)?;
    consumer.subscribe(topics)?;
    Ok(consumer)
}
