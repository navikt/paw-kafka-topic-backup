macro_rules! data_table {
    () => {
        "data_v2"
    };
}
macro_rules! hwm_table {
    () => {
        "hwm"
    };
}

pub const INSERT_DATA: &str = concat!(
    "INSERT INTO ",
    data_table!(),
    " (",
    "kafka_topic, kafka_partition, kafka_offset, ",
    "timestamp, headers, record_key, record_value",
    ") VALUES ($1, $2, $3, $4, $5, $6, $7)"
);

pub const QUERY_HWM: &str = concat!(
    "SELECT hwm FROM ",
    hwm_table!(),
    " WHERE topic = $1 AND partition = $2"
);

pub const INSERT_HWM: &str = concat!(
    "INSERT INTO ",
    hwm_table!(),
    " (topic, partition, hwm) ",
    "VALUES ($1, $2, $3)"
);

pub const UPDATE_HWM: &str = concat!(
    "UPDATE ",
    hwm_table!(),
    " SET hwm = $3 WHERE topic = $1 AND partition = $2 AND hwm < $3"
);
