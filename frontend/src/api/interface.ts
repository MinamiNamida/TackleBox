/*
====================
Authenticated User,
User Payload and Response
====================
*/

export interface LoginPayload {
    username: string,
    password: string,
}

export interface LoginResponse {
    user_id: string,
    token: string,
    token_type: 'Bearer'
};

export interface RegisterPayload {
    username: string,
    password: string,
    email: string | null
}


export interface RegisterResponse {
    user_id: string,
    token: string,
    token_type: 'Bearer',
};

export interface GetUserResponse {
    user_id: string,
    username: string,
    created_at: string
};

/*
====================
Agent Manager Payload
====================
*/

export type AgentPolicy = 'Idle' | 'AutoJoin' | 'AutoNewAndJoin';
export type AgentStatus = 'Idle' | 'Running' | 'Ready';

export interface GetAgentPayload {
    agent_id: string
}

export interface GetAgentResponse {
    agent_id: string,
    name: string,
    game_type_id: string,
    game_type_name: string,
    owner_id: string,
    owner_name: string,
    version: string,
    description: string | null,
    played_games: number,
    won_games: number,
    status: AgentStatus,
    policy: AgentPolicy,
    created_at: string,
    updated_at: string,
}

export interface NewAgentPayload {
    name: string,
    game_type_id: string,
    version: string,
    description: string | null,
    policy: AgentPolicy,
}

export interface UpdateAgentPayload {
    agent_id: string,
    name: string,
    game_type_id: string,
    version: string,
    description: string | null,
    policy: AgentPolicy,
}

export interface DeleteAgentPayload {
    agent_id: string
}

/*
====================
Match Manager Payload
====================
*/

export type MatchStatus = 'Pending' | 'Running' | 'Completed' | 'Cancelled';

export interface GetMatchPayload {
    match_id: string
}

export interface GetMatchResponse {
    match_id: string,
    match_name: string,
    creater_id: string,
    creater_name: string,
    winner_id: string | null,
    winner_agent_name: string | null,
    game_type_id: string,
    game_type_name: string,
    total_games: number,
    password: string | null,
    max_slots: number,
    min_slots: number,
    status: MatchStatus,
    start_time: string,
    end_time: string | null,
}

export interface GetOnlineMatchResponse {
    match_id: string,
    match_name: string,
    creater_id: string,
    creater_name: string,
    game_type_id: string,
    game_type_name: string,
    with_password: boolean,
    max_slots: number,
    min_slots: number,
    current_slots: number
    status: MatchStatus,
    start_time: string,
    total_games: number,
}

export interface NewMatchPayload {
    name: string,
    game_type_id: string,
    total_games: number,
    with_agent_ids: string[],
    password: string | null
}

export interface UpdateMatchPayload {
    name: string,
    game_type_id: string,
    total_game: number,
    password: string | null
}

export interface JoinMatchPayload {
    match_id: string,
    agent_ids: string[],
    password: string | null,
}

export interface LevaeMatchPayload {
    match_id: string,
    agent_id: string[],
}

export interface GetMatchLogsPayload {
    match_id: string
}

export interface TurnLogResponse {
    turn_id: string,
    match_id: string,
    log: string,
    i_turn: number,
    score_deltas: Record<string, number>
}

export interface GetParticipantsPayload {
    match_id: string
}

export interface GetParticipantsResponse {
    match_id: string,
    match_name: string,
    agent_id: string,
    agent_name: string,
}

/*
====================
GameType Payload
====================
*/

export interface GetGameTypeResponse {
    game_type_id: string,
    name: string,
    sponsor: string,
    description: string | null,
    min_slots: number,
    max_slots: number,
}

export interface GetStatsResponse {
    game_type_id: string,
    game_type_name: string
    agent_id: string,
    agent_name: string,
    rank: number,
    updated_time: string,
}