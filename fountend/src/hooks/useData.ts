import { useQuery } from "@tanstack/react-query"
import { fecthGetGameTypes, fetchGetAgent, fetchGetAgents, fetchGetMatch, fetchGetMatches, fetchGetOnlineMatch, fetchGetStats, fetchTurnLog } from "../api/fetch";

export const useAgents = () => {
    return useQuery({
        queryKey: ['agents'],
        queryFn: fetchGetAgents,
    });
};

// export const useAgent = (name: string) => {
//     return useQuery({
//         queryKey: ['agent', name],
//         queryFn: () => fetchGetAgent({ name })
//     });
// };

export const useMatches = () => {
    return useQuery({
        queryKey: ['matches'],
        queryFn: fetchGetMatches,
    });
};

// export const useMatch = (name: string) => {
//     return useQuery({
//         queryKey: ['match', name],
//         queryFn: () => fetchGetMatch({ name })
//     });
// };

export const useTurnLog = (name: string) => {
    return useQuery({
        queryKey: ['turn', name],
        queryFn: () => fetchTurnLog({ name }),
    });
};

export const useOnlineMatches = () => {
    return useQuery({
        queryKey: ['online_match'],
        queryFn: fetchGetOnlineMatch,
        staleTime: 30 * 1000
    });
}

export const useStats = () => {
    return useQuery({
        queryKey: ['stats'],
        queryFn: fetchGetStats,
        staleTime: 30 * 60 * 1000
    });
};

export const useGameType = () => {
    return useQuery({
        queryKey: ['game_type'],
        queryFn: fecthGetGameTypes,
    });
} 