use std::{error::Error, sync::Arc};

use crate::{
    app_state::AppState,
    database::hwm_statements::{get_hwm, insert_hwm},
};
use log::{error, info};
use rdkafka::{
    ClientContext, Offset,
    consumer::{BaseConsumer, Consumer, ConsumerContext, Rebalance},
    topic_partition_list::TopicPartitionListElem,
};
use sqlx::PgPool;

pub struct Hwm {
    topic: String,
    partition: i32,
    hwm: i64,
}

impl Hwm {
    fn seek_to_rdkafka_offset(&self) -> Offset {
        match self.hwm {
            -1 => Offset::Beginning,
            _ => Offset::Offset(self.hwm + 1), //HWM er sist leste melding, i seek_to skal vi ha neste melding vi vil lese.
        }
    }
}

pub struct Topic {
    name: String,
    partition: i32,
}

pub struct HwmRebalanceHandler {
    pub pg_pool: PgPool,
    pub app_state: Arc<AppState>,
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
                    "HWM for {}::{} not found, inserting {} as HWM in DB",
                    topic.name, topic.partition, DEFAULT_HWM
                );
                insert_hwm(&mut tx, &topic.name, topic.partition, DEFAULT_HWM).await?;
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
        tx.commit().await?;
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

                let hwm = futures::executor::block_on(self.get_hwms(topics)).unwrap();
                hwm.iter().for_each(|hwm| {
                    info!(
                        "Assigned: topic: {}, partition: {}, hwm: {}",
                        hwm.topic, hwm.partition, hwm.hwm
                    );
                    base_consumer
                        .seek(
                            &hwm.topic,
                            hwm.partition,
                            hwm.seek_to_rdkafka_offset(),
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
                self.app_state.set_is_alive(false);
            }
        }
    }
}
