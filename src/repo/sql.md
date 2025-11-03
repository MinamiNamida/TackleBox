V1
```sql
BEGIN;

-- ------------------------------
-- 1. DROP ALL TABLES AND TYPES
-- ------------------------------
DROP VIEW IF EXISTS v_readable_participations;
DROP VIEW IF EXISTS v_readable_turns;
DROP VIEW IF EXISTS v_readable_matches;
DROP VIEW IF EXISTS v_readable_agents;

DROP TABLE IF EXISTS STATS;
DROP TABLE IF EXISTS PARTICIPATIONS;
DROP TABLE IF EXISTS TURNS;
DROP TABLE IF EXISTS MATCHES;
DROP TABLE IF EXISTS AGENTS;
DROP TABLE IF EXISTS "users";
DROP TABLE IF EXISTS GAMETYPES;
-- Drop the custom enum type if it exists
DROP TYPE IF EXISTS MATCH_STATUS;
-- ------------------------------
-- 2. CREATE TABLES (In dependency order)
-- ------------------------------

-- GAMETYPE (Independent)
-- Note: 'name' is the PK and is NOT database-generated.
CREATE TABLE GAMETYPES (
    name        VARCHAR(255) PRIMARY KEY, -- PK (string name) - Manual input
    description TEXT
);

---

-- USER (id is DB-generated)
CREATE TABLE "users" (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- PK (uuid id) - DB Generated
    username      VARCHAR(100) UNIQUE NOT NULL, -- UK
    password_hash VARCHAR(255) NOT NULL,
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

---

-- AGENT (id is DB-generated)
CREATE TABLE AGENTS (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- PK (uuid id) - DB Generated
    name          VARCHAR(255) NOT NULL,
    owner_id      UUID NOT NULL REFERENCES "users" (id),      -- FK (uuid owner_id)
    
    -- FK for GAMETYPE(name) - Assuming you want to link to the name string
    game_type     VARCHAR(255) NOT NULL REFERENCES GAMETYPES (name), 
    
    version       VARCHAR(50) NOT NULL,
    description   TEXT,
    played_games  INT DEFAULT 0 NOT NULL,
    won_games     INT DEFAULT 0 NOT NULL,

    UNIQUE (owner_id, name) 
);

---

-- MATCH (id is DB-generated)
CREATE TYPE MATCH_STATUS AS ENUM ('Pending', 'Running', 'Completed', 'Cancelled');
CREATE TABLE MATCHES (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- PK (uuid id) - DB Generated
    name           VARCHAR(255) NOT NULL,
    password       VARCHAR(16),
    -- FK for GAMETYPE(name)
    game_type      VARCHAR(255) NOT NULL REFERENCES GAMETYPES (name), 
    
    total_games    INT NOT NULL,
    creater_id     UUID NOT NULL REFERENCES "users" (id),   -- FK (uuid creater_id)
    winner_id      UUID REFERENCES AGENTS (id),             -- FK (uuid winner_id), Nullable
    start_time     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    end_time       TIMESTAMP WITH TIME ZONE,
    status         MATCH_STATUS NOT NULL
);

---

-- TURN (id is DB-generated)
CREATE TABLE TURNS (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- PK (uuid id) - DB Generated
    match_id       UUID NOT NULL REFERENCES MATCHES (id),        -- FK (uuid match_id)
    log            JSONB NOT NULL,
    i_turn         INT NOT NULL,
    score_deltas   JSONB NOT NULL,
    start_time     TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time       TIMESTAMP WITH TIME ZONE NOT NULL,
    
    UNIQUE (match_id, i_turn) 
);

---

-- PARTICIPATION (Composite PKs are FKs, so no DB-generated UUID needed)
CREATE TABLE PARTICIPATIONS (
    match_id       UUID NOT NULL REFERENCES MATCHES (id),      -- PK,FK
    agent_id       UUID NOT NULL REFERENCES AGENTS (id),      -- PK,FK
    
    PRIMARY KEY (match_id, agent_id)
);

---

-- STATS (Composite PKs are FKs, so no DB-generated UUID needed)
CREATE TABLE STATS (
    game_type      VARCHAR(255) REFERENCES GAMETYPES (name),  -- PK,FK
    agent_id       UUID REFERENCES AGENTS (id),               -- PK,FK
    rank           INT NOT NULL,
    updated_time   TIMESTAMP WITH TIME ZONE NOT NULL,
    
    PRIMARY KEY (game_type, agent_id)
);

CREATE VIEW v_readable_agents AS
SELECT
    -- 原始 ID 字段：保留
    A.id AS agent_id,
    A.owner_id,
    
    -- 基础 Agent 信息
    A.name AS agent_name_base,
    A.game_type,
    A.version,
    A.description,
    A.played_games,
    A.won_games,

    -- 引入 User 的关键信息，并明确别名为 owner_username
    U.username AS owner_username,
    
    -- 核心字段：生成易读的标识符 (agentname@username)
    CONCAT(A.name, '@', U.username) AS readable_agent_name
FROM
    AGENTS A
INNER JOIN
    users U ON A.owner_id = U.id;

CREATE VIEW v_readable_matches AS
SELECT
    -- 原始 ID 字段：保留
    M.id AS match_id,
    M.creater_id,
    M.winner_id, -- 胜者 Agent 的 UUID
    
    -- 核心字段 1：易读的比赛标识符 (matchname@creater_username)
    CONCAT(M.name, '@', U_CREATER.username) AS readable_match_name,
    
    -- 可读字段 1：创建者用户名
    U_CREATER.username AS creater_username,
    
    -- 核心字段 2：胜者 Agent 的复合可读名称 (AgentName@UserName)
    CASE 
        -- 仅当 M.winner_id 不为 NULL 时才进行拼接
        WHEN M.winner_id IS NOT NULL THEN CONCAT(A_WINNER.name, '@', U_WINNER.username) 
        ELSE NULL 
    END AS winner_agent_readable_name,
    
    -- 其他比赛基础信息
    M.name AS match_name_base,
    M.password,
    M.game_type,
    M.total_games,
    M.start_time,
    M.end_time,
    M.status

FROM
    MATCHES M
INNER JOIN
    users U_CREATER ON M.creater_id = U_CREATER.id  -- 获取创建者用户名
LEFT JOIN
    AGENTS A_WINNER ON M.winner_id = A_WINNER.id    -- 获取胜者 Agent (可能为 NULL)
LEFT JOIN
    users U_WINNER ON A_WINNER.owner_id = U_WINNER.id -- 获取胜者 Agent 的所有者用户名 (可能为 NULL)
ORDER BY M.start_time DESC; -- （可选：按时间排序）

CREATE VIEW v_readable_turns AS
SELECT
    T.*,
    R.readable_match_name
FROM
    TURNS T
INNER JOIN
    v_readable_matches R ON T.match_id = R.match_id;


CREATE VIEW v_readable_participations AS
SELECT
    P.match_id,
    P.agent_id,
    M.readable_match_name,
    A.readable_agent_name
FROM
    PARTICIPATIONS P
INNER JOIN
    v_readable_matches M ON P.match_id = M.match_id
INNER JOIN
    v_readable_agents A ON P.agent_id = A.agent_id;

-- ------------------------------
-- 3. FINALIZE TRANSACTION
-- ------------------------------
COMMIT;
```


V2
```sql
BEGIN;

-- ------------------------------
-- 1. DROP ALL TABLES AND TYPES
-- ------------------------------

DROP TABLE IF EXISTS STATS;
DROP TABLE IF EXISTS PARTICIPATANTS;
DROP TABLE IF EXISTS TURNS;
DROP TABLE IF EXISTS MATCHES;
DROP TABLE IF EXISTS AGENTS;
DROP TABLE IF EXISTS "users";
DROP TABLE IF EXISTS GAMETYPES;
-- Drop the custom enum type if it exists
DROP TYPE IF EXISTS MATCH_STATUS;
-- ------------------------------
-- 2. CREATE TABLES (In dependency order)
-- ------------------------------

-- GAMETYPE (Independent)
-- Note: 'name' is the PK and is NOT database-generated.
CREATE TABLE GAMETYPES (
    name        VARCHAR(255) PRIMARY KEY, -- PK (string name) - Manual input
    description TEXT
);

---

-- USER (id is DB-generated)
CREATE TABLE "users" (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- PK (uuid id) - DB Generated
    username      VARCHAR(100) UNIQUE NOT NULL, -- UK
    password_hash VARCHAR(255) NOT NULL,
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

---

-- AGENT (id is DB-generated)
CREATE TABLE AGENTS (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- PK (uuid id) - DB Generated
    name          VARCHAR(255) NOT NULL,
    owner_id      UUID NOT NULL REFERENCES "users" (id),      -- FK (uuid owner_id)
    
    -- FK for GAMETYPE(name) - Assuming you want to link to the name string
    game_type     VARCHAR(255) NOT NULL REFERENCES GAMETYPES (name), 
    
    version       VARCHAR(50) NOT NULL,
    description   TEXT,
    played_games  INT DEFAULT 0 NOT NULL,
    won_games     INT DEFAULT 0 NOT NULL,

    UNIQUE (owner_id, name) 
);

---

-- MATCH (id is DB-generated)
CREATE TYPE MATCH_STATUS AS ENUM ('Pending', 'Running', 'Completed', 'Cancelled');
CREATE TABLE MATCHES (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- PK (uuid id) - DB Generated
    name           VARCHAR(255) NOT NULL,
    password       VARCHAR(16),
    -- FK for GAMETYPE(name)
    game_type      VARCHAR(255) NOT NULL REFERENCES GAMETYPES (name), 
    
    total_games    INT NOT NULL,
    creater_id     UUID NOT NULL REFERENCES "users" (id),   -- FK (uuid creater_id)
    winner_id      UUID REFERENCES AGENTS (id),             -- FK (uuid winner_id), Nullable
    start_time     TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    end_time       TIMESTAMP WITH TIME ZONE,
    status         MATCH_STATUS NOT NULL
);

---

-- TURN (id is DB-generated)
CREATE TABLE TURNS (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(), -- PK (uuid id) - DB Generated
    match_id       UUID NOT NULL REFERENCES MATCHES (id),        -- FK (uuid match_id)
    log            JSONB NOT NULL,
    i_turn         INT NOT NULL,
    score_deltas   JSONB NOT NULL,
    start_time     TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time       TIMESTAMP WITH TIME ZONE NOT NULL,
    
    UNIQUE (match_id, i_turn) 
);

---

-- PARTICIPATION (Composite PKs are FKs, so no DB-generated UUID needed)
CREATE TABLE PARTICIPATIONS (
    match_id       UUID NOT NULL REFERENCES MATCHES (id),      -- PK,FK
    agent_id       UUID NOT NULL REFERENCES AGENTS (id),      -- PK,FK
    
    PRIMARY KEY (match_id, agent_id)
);

---

-- STATS (Composite PKs are FKs, so no DB-generated UUID needed)
CREATE TABLE STATS (
    game_type      VARCHAR(255) REFERENCES GAMETYPES (name),  -- PK,FK
    agent_id       UUID REFERENCES AGENTS (id),               -- PK,FK
    rank           INT NOT NULL,
    updated_time   TIMESTAMP WITH TIME ZONE NOT NULL,
    
    PRIMARY KEY (game_type, agent_id)
);

CREATE VIEW v_readable_agents AS
SELECT
    -- 原始 ID 字段：保留
    A.id AS agent_id,
    A.owner_id,
    
    -- 基础 Agent 信息
    A.name AS agent_name_base,
    A.game_type,
    A.version,
    A.description,
    A.played_games,
    A.won_games,

    -- 引入 User 的关键信息，并明确别名为 owner_username
    U.username AS owner_username,
    
    -- 核心字段：生成易读的标识符 (agentname@username)
    CONCAT(A.name, '@', U.username) AS readable_agent_name
FROM
    AGENTS A
INNER JOIN
    users U ON A.owner_id = U.id;

CREATE VIEW v_readable_matches AS
SELECT
    -- 原始 ID 字段：保留
    M.id AS match_id,
    M.creater_id,
    M.winner_id, -- 胜者 Agent 的 UUID
    
    -- 核心字段 1：易读的比赛标识符 (matchname@creater_username)
    CONCAT(M.name, '@', U_CREATER.username) AS readable_match_name,
    
    -- 可读字段 1：创建者用户名
    U_CREATER.username AS creater_username,
    
    -- 核心字段 2：胜者 Agent 的复合可读名称 (AgentName@UserName)
    CASE 
        -- 仅当 M.winner_id 不为 NULL 时才进行拼接
        WHEN M.winner_id IS NOT NULL THEN CONCAT(A_WINNER.name, '@', U_WINNER.username) 
        ELSE NULL 
    END AS winner_agent_readable_name,
    
    -- 其他比赛基础信息
    M.name AS match_name_base,
    M.password,
    M.game_type,
    M.total_games,
    M.start_time,
    M.end_time,
    M.status

FROM
    MATCHES M
INNER JOIN
    users U_CREATER ON M.creater_id = U_CREATER.id  -- 获取创建者用户名
LEFT JOIN
    AGENTS A_WINNER ON M.winner_id = A_WINNER.id    -- 获取胜者 Agent (可能为 NULL)
LEFT JOIN
    users U_WINNER ON A_WINNER.owner_id = U_WINNER.id -- 获取胜者 Agent 的所有者用户名 (可能为 NULL)
ORDER BY M.start_time DESC; -- （可选：按时间排序）

CREATE VIEW v_readable_turns AS
SELECT
    T.*,
    R.readable_match_name
FROM
    TURNS T
INNER JOIN
    v_readable_matches R ON T.match_id = R.match_id;


CREATE VIEW v_readable_participations AS
SELECT
    P.match_id,
    P.agent_id,
    M.readable_match_name,
    A.readable_agent_name
FROM
    PARTICIPATIONS P
INNER JOIN
    v_readable_matches M ON P.match_id = M.match_id
INNER JOIN
    v_readable_agents A ON P.agent_id = A.agent_id;

-- ------------------------------
-- 3. FINALIZE TRANSACTION
-- ------------------------------
COMMIT;
```