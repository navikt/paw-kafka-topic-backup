use std::{error::Error, process::exit, sync::Arc};

use crate::database::hwm_statements::{get_hwm, insert_hwm};
use log::{error, info};
use rdkafka::{
    ClientContext, Offset,
    consumer::{BaseConsumer, Consumer, ConsumerContext, Rebalance},
    topic_partition_list::TopicPartitionListElem,
};
use sqlx::PgPool;
use tokio::runtime::{Handle, Runtime};

pub struct Hwm {
    topic: String,
    partition: i32,
    hwm: i64,
}

impl Hwm {
    fn rdkafka_offset(&self) -> Offset {
        match self.hwm {
            -1 => Offset::Beginning,
            _ => Offset::Offset(self.hwm),
        }
    }
}

pub struct Topic {
    name: String,
    partition: i32,
}

pub struct HwmRebalanceHandler {
    pub pg_pool: Arc<PgPool>,
}

impl Default for HwmRebalanceHandler {
    fn default() -> Self {
        panic!("Default not implemented for HwmRebalanceHandler");
    }
}
const DEFAULT_HWM: i64 = -1;
impl HwmRebalanceHandler {
    async fn get_hwms(&self, topics: Vec<Topic>) -> Result<Vec<Hwm>, Box<dyn Error>> {
        let mut tx = self.pg_pool.begin().await?;
        let mut hwms = Vec::new();
        for topic in topics {
            let hwm = get_hwm(&mut tx, &topic.name, topic.partition).await?;
            let hwm = if hwm.is_none() {
                info!(
                    "HWM for {}::{} not found, inserting -1 as HWM in DB",
                    topic.name, topic.partition
                );
                insert_hwm(&mut tx, topic.name.clone(), topic.partition, DEFAULT_HWM).await?;
                DEFAULT_HWM
            } else {
                hwm.unwrap()
            };
            hwms.push(Hwm {
                topic: topic.name,
                partition: topic.partition,
                hwm: hwm,
            });
        }
        Ok(hwms)
    }
}

impl ClientContext for HwmRebalanceHandler {}

impl ConsumerContext for HwmRebalanceHandler {
    fn post_rebalance(&self, base_consumer: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        match rebalance {
            Rebalance::Assign(topic_partitions) => {
                let topics = topic_partitions
                    .elements()
                    .iter()
                    .map(|elem: &TopicPartitionListElem| Topic {
                        name: elem.topic().to_string(),
                        partition: elem.partition(),
                    })
                    .collect();
                let runtime = Runtime::new().unwrap();
                let hwm = Handle::block_on(runtime.handle(), self.get_hwms(topics)).unwrap();
                hwm.iter().for_each(|hwm| {
                    info!(
                        "Assigned: topic: {}, partition: {}, hwm: {}",
                        hwm.topic, hwm.partition, hwm.hwm
                    );
                    base_consumer
                        .seek(
                            &hwm.topic,
                            hwm.partition,
                            hwm.rdkafka_offset(),
                            std::time::Duration::from_secs(10),
                        )
                        .unwrap();
                });
            }
            Rebalance::Revoke(_) => {
                info!("Topic partitions revoked")
            }
            Rebalance::Error(e) => {
                error!("Rebalance error: {}", e);
                exit(1);
            }
        }
    }
}
