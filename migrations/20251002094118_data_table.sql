CREATE TABLE IF NOT EXISTS data_v2 (
    id BIGSERIAL PRIMARY KEY,
    kafka_topic VARCHAR(255) NOT NULL,
    kafka_partition SMALLINT NOT NULL,
    kafka_offset BIGINT NOT NULL,
    timestamp TIMESTAMP(3) WITH TIME ZONE NOT NULL,
    headers JSONB,
    record_key BYTEA,
    record_value BYTEA,
    UNIQUE(kafka_topic, kafka_partition, kafka_offset)
);
