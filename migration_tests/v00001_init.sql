
CREATE TABLE IF NOT EXISTS init_table (
   filename VARCHAR(100) NOT NULL,
   hash VARCHAR(64) NOT NULL,
   executed_at TIMESTAMP WITH TIME ZONE,
   PRIMARY KEY (filename)
)

