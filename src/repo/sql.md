V2
```sql
BEGIN;

-- ------------------------------
-- 1. DROP ALL TABLES AND TYPES
-- ------------------------------

DROP TABLE IF EXISTS STATS;
DROP TABLE IF EXISTS PARTICIPANTS;
DROP TABLE IF EXISTS TURNS;
DROP TABLE IF EXISTS MATCHES;
DROP TABLE IF EXISTS AGENTS;
DROP TABLE IF EXISTS "users";
DROP TABLE IF EXISTS GAMETYPES;
DROP TYPE IF EXISTS MATCH_STATUS;
DROP TYPE IF EXISTS AGENT_POLICY;
DROP TYPE IF EXISTS AGENT_STATUS;
-- ------------------------------
-- 2. CREATE TABLES (In dependency order)
-- ------------------------------

-- GAMETYPE (Independent)
-- Note: 'name' is the PK and is NOT database-generated.
CREATE TABLE GAMETYPES (
    game_type_id  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name          VARCHAR(255) NOT NULL, -- PK (string name) - Manual input
    sponsor       VARCHAR(255) NOT NULL,
    max_slots     INT NOT NULL CHECK (max_slots >= min_slots),
    min_slots     INT NOT NULL CHECK (min_slots >= 0),
    description   TEXT
);

---

-- USER (id is DB-generated)
CREATE TABLE "users" (
    user_id            UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- PK (uuid id) - DB Generated
    username      VARCHAR(100) UNIQUE NOT NULL, -- UK
    password_hash VARCHAR(255) NOT NULL,
    email         VARCHAR(255) NOT NULL,
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TYPE AGENT_POLICY AS ENUM ('Idle', 'AutoJoin', 'AutoNewAndJoin');
CREATE TYPE AGENT_STATUS AS ENUM ('Idle', 'Ready', 'Running', 'Decommissioned');

-- AGENT (id is DB-generated)
CREATE TABLE AGENTS (
    agent_id      UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- PK (uuid id) - DB Generated
    name          VARCHAR(255) NOT NULL,
    owner_id      UUID NOT NULL REFERENCES "users" (user_id),      -- FK (uuid owner_id)
    
    -- FK for GAMETYPE(name) - Assuming you want to link to the name string
    game_type_id  UUID NOT NULL REFERENCES GAMETYPES (game_type_id), 
    version       VARCHAR(50) NOT NULL,
    description   TEXT,
    played_games  INT DEFAULT 0 NOT NULL CHECK (played_games >= 0),
    won_games     INT DEFAULT 0 NOT NULL CHECK (won_games >= 0),
    policy        AGENT_POLICY NOT NULL DEFAULT 'Idle',
    status        AGENT_STATUS NOT NULL DEFAULT 'Idle',
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (owner_id, name) 
);

---

-- MATCH (id is DB-generated)
CREATE TYPE MATCH_STATUS AS ENUM ('Pending', 'Running', 'Completed', 'Cancelled');
CREATE TABLE MATCHES (
    match_id       UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- PK (uuid id) - DB Generated
    name           VARCHAR(255) NOT NULL,
    password       VARCHAR(16),
    -- FK for GAMETYPE(name)
    game_type_id   UUID NOT NULL REFERENCES GAMETYPES (game_type_id), 
    total_games    INT NOT NULL,
    creater_id     UUID NOT NULL REFERENCES "users" (user_id),   -- FK (uuid creater_id)
    winner_id      UUID REFERENCES AGENTS (agent_id),             -- FK (uuid winner_id), Nullable
    start_time     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    end_time       TIMESTAMP WITH TIME ZONE,
    status         MATCH_STATUS NOT NULL DEFAULT 'Pending'
);

---

-- TURN (id is DB-generated)
CREATE TABLE TURNS (
    turn_id        UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- PK (uuid id) - DB Generated
    match_id       UUID NOT NULL REFERENCES MATCHES (match_id),        -- FK (uuid match_id)
    log            JSONB NOT NULL,
    i_turn         INT NOT NULL,
    score_deltas   JSONB NOT NULL,
    start_time     TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time       TIMESTAMP WITH TIME ZONE NOT NULL,
    
    UNIQUE (match_id, i_turn) 
);

---

-- PARTICIPATION (Composite PKs are FKs, so no DB-generated UUID needed)
CREATE TABLE PARTICIPANTS (
    match_id       UUID NOT NULL REFERENCES MATCHES (match_id),      -- PK,FK
    agent_id       UUID NOT NULL REFERENCES AGENTS (agent_id),      -- PK,FK
    
    PRIMARY KEY (match_id, agent_id)
);

---

-- STATS (Composite PKs are FKs, so no DB-generated UUID needed)
CREATE TABLE STATS (
    game_type_id   UUID NOT NULL REFERENCES GAMETYPES (game_type_id),  -- PK,FK
    agent_id       UUID NOT NULL REFERENCES AGENTS (agent_id),               -- PK,FK
    rank           INT NOT NULL,
    updated_time   TIMESTAMP WITH TIME ZONE NOT NULL,

    PRIMARY KEY (game_type_id, agent_id)
);


COMMIT;
```

data
```sql
INSERT INTO gametypes (name, sponsor, max_slots, min_slots) VALUES ('leduc-holdem', 'rlcard', 2, 2);

```