CREATE TABLE hwm (
    topic VARCHAR(255) NOT NULL,
    partition SMALLINT NOT NULL,
    hwm BIGINT NOT NULL,
    PRIMARY KEY (topic, partition)
);