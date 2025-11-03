import apiClient from './apiClient';
import type { DeleteAgentPayload, GetAgentPayload, GetAgentResponse, GetGameTypeResponse, GetMatchPayload, GetMatchResponse, GetOnlineMatchResponse, GetParticipantsPayload, GetParticipantsResponse, GetStatsResponse, GetUserResponse, JoinMatchPayload, LevaeMatchPayload, LoginPayload, LoginResponse, NewAgentPayload, NewMatchPayload, RegisterPayload, RegisterResponse, TurnLogResponse, UpdateAgentPayload, UpdateMatchPayload } from './interface';

/*
====================
Authenticated User,
User Payload and Response
====================
*/



export const fetchLogin = async (payload: LoginPayload): Promise<LoginResponse> => {
    const response = await apiClient.post('/auth/login', payload);
    return response.data;
};

export const fetchRegister = async (payload: RegisterPayload): Promise<RegisterResponse> => {
    const resp = await apiClient.post('/auth/register', payload);
    return resp.data;
};

export const fetchGetUser = async (): Promise<GetUserResponse> => {
    const response = await apiClient.get('/auth/me');
    return response.data;
}

/*
====================
Agent Manager Payload
====================
*/


export const fetchGetAgents = async (): Promise<GetAgentResponse[]> => {
    const response = await apiClient.get('/agent/agents');
    return response.data;
}

export const fetchGetAgent = async (paylaod: GetAgentPayload): Promise<GetAgentResponse> => {
    const resp = await apiClient.post('/agent/get', paylaod);
    return resp.data;
}

export const fetchNewAgent = async (payload: NewAgentPayload): Promise<void> => {
    const response = await apiClient.post('/agent/new', payload);
}


export const fetchDeleteAgent = async (paylaod: DeleteAgentPayload): Promise<void> => {
    await apiClient.post('/agent/delete', paylaod);
}


export const fetchUpdateAgent = async (paylaod: UpdateAgentPayload): Promise<void> => {
    await apiClient.post('/agent/update', paylaod);
}

/*
====================
Match Manager Payload
====================
*/

export const fetchGetMatches = async (): Promise<GetMatchResponse[]> => {
    const response = await apiClient.get('/match/matches');
    return response.data;
}

export const fetchGetMatch = async (paylaod: GetMatchPayload): Promise<GetMatchResponse> => {
    const response = await apiClient.post('/match/get', paylaod);
    return response.data;
}

export const fetchGetOnlineMatch = async (): Promise<GetOnlineMatchResponse[]> => {
    const response = await apiClient.get('/match/search');
    return response.data;
}

export const fetchNewMatch = async (payload: NewMatchPayload): Promise<void> => {
    const response = await apiClient.post('/match/new', payload);
}


export const fetchUpdateMatch = async (paylaod: UpdateMatchPayload): Promise<void> => {
    await apiClient.post('/match/update', paylaod);
}

export const fetchJoinMatch = async (paylaod: JoinMatchPayload) => {
    await apiClient.post('/match/join', paylaod);
}

export const fetchLeaveMatch = async (paylaod: LevaeMatchPayload) => {
    await apiClient.post('/match/leave', paylaod);
}

export const fetchTurnLog = async (payload: GetMatchPayload): Promise<TurnLogResponse[]> => {
    const response = await apiClient.post('/match/turns', payload);
    return response.data;
}


export const fetchGetParticipations = async (payload: GetParticipantsPayload): Promise<GetParticipantsResponse[]> => {
    const resp = await apiClient.post('/match/participants', payload)
    return resp.data;
}

/*
====================
Stats Payload
====================
*/

export const fetchGetStats = async (): Promise<GetStatsResponse[]> => {
    const response = await apiClient.get('/stat');
    return response.data;
}

/*
====================
GameType Payload
====================
*/

export const fecthGetGameTypes = async (): Promise<GetGameTypeResponse[]> => {
    const resp = await apiClient.get('/match/gametypes');
    return resp.data
};
