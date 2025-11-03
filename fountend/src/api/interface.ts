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
    token: string,
    token_type: 'Bearer'
};

export interface RegisterPayload {
    username: string,
    password: string,
    email: string | null
}


export interface RegisterResponse {
    token: string,
    token_type: 'Bearer',
};

export interface GetUserResponse {
    username: string,
    created_at: string
};

/*
====================
Agent Manager Payload
====================
*/

export interface GetAgentPayload {
    name: string
}

export interface GetAgentResponse {
    id: string,
    name: string,
    version: string,
    game_type: string,
    description: string | null,
    played_games: number,
    won_games: number,
}

export interface NewAgentPayload {
    name: string,
    game_type: string,
    version: string,
    description: string | null,
}

export interface UpdateAgentPayload {
    name: string,
    game_type: string,
    version: string,
    description: string | null
}

export interface DeleteAgentPayload {
    name: string
}

/*
====================
Match Manager Payload
====================
*/


export interface GetMatchPayload {
    name: string
}

export interface GetMatchResponse {
    id: string,
    name: string,
    game_type: string,
    total_games: number,
    creater_name: string,
    winner_agent_name: string,
    start_time: string,
    end_time: string,
    status: 'Pending' | 'Completed' | 'Cancelled' | 'Running'
}

export interface GetOnlineMatchResponse {
    match_id: string,
    match_name: string,
    game_type: string,
    creater_name: string,
    with_password: boolean,
    status: 'Pending' | 'Completed' | 'Cancelled' | 'Running'
}

export interface NewMatchPayload {
    name: string,
    game_type: string,
    total_games: number,
    with_agent_names: string[],
    password: string | null
}

export interface UpdateMatchPayload {
    name: string,
    game_type: string,
    total_game: number,
    password: string | null
}

export interface JoinMatchPayload {
    match_name: string,
    agent_name: string,
}

export interface LevaeMatchPayload {
    match_name: string,
    agent_name: string,
}

export interface GetMatchLogsPayload {
    match_name: string
}

export interface TurnLogResponse {
    id: string,
    match_name: string,
    log: string,
    i_turn: number,
    score_deltas: Record<string, number>
}

export interface GetParticipantsPayload {
    match_name: string
}

export interface GetParticipantsResponse {
    match_name: string,
    agent_name: string,
}

/*
====================
GameType Payload
====================
*/

export interface GetGameTypeResponse {
    game_type: string,
    description: string | null
}

export interface GetStatsResponse {
    game_type: string,
    agent_name: string,
    rank: number,
    updated_time: string,
}