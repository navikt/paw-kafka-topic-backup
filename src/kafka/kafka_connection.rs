use std::{error::Error, sync::Arc};

use rdkafka::consumer::{Consumer, StreamConsumer};
use sqlx::PgPool;

use crate::{app_state::AppState, kafka::{config::ApplicationKafkaConfig, hwm::HwmRebalanceHandler}};

pub fn create_kafka_consumer(
    app_state: Arc<AppState>,
    pg_pool: PgPool,
    app_config: ApplicationKafkaConfig,
    topics: &[&str],
) -> Result<StreamConsumer<HwmRebalanceHandler>, Box<dyn Error>> {
    let config = app_config.rdkafka_config()?;
    let context = HwmRebalanceHandler { pg_pool, app_state };
    let consumer: StreamConsumer<HwmRebalanceHandler> = config.create_with_context(context)?;
    consumer.subscribe(topics)?;
    Ok(consumer)
}
