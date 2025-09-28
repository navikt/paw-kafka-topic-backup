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

pub const CREATE_DATA_TABLE: &str = concat!(
    "CREATE TABLE IF NOT EXISTS ",
    data_table!(),
    " (",
    "id BIGSERIAL PRIMARY KEY,",
    "kafka_topic VARCHAR(255) NOT NULL,",
    "kafka_partition SMALLINT NOT NULL,",
    "kafka_offset BIGINT NOT NULL,",
    "timestamp TIMESTAMP(3) WITH TIME ZONE NOT NULL,",
    "headers JSONB,",
    "record_key BYTEA,",
    "record_value BYTEA,",
    "UNIQUE(kafka_topic, kafka_partition, kafka_offset)",
    ");"
);

pub const INSERT_DATA: &str = concat!(
    "INSERT INTO ",
    data_table!(),
    " (",
    "kafka_topic, kafka_partition, kafka_offset, ",
    "timestamp, headers, record_key, record_value",
    ") VALUES ($1, $2, $3, $4, $5, $6, $7)"
);

pub const CREATE_HWM_TABLE: &str = concat!(
    "CREATE TABLE IF NOT EXISTS ",
    hwm_table!(),
    " (",
    "topic VARCHAR(255) NOT NULL,",
    "partition SMALLINT NOT NULL,",
    "hwm BIGINT NOT NULL,",
    "PRIMARY KEY (topic, partition)",
    ");"
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
