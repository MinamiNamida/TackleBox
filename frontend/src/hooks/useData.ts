import { useQuery } from "@tanstack/react-query"
import { fecthGetGameTypes, fetchGetAgents, fetchGetMatches, fetchGetOnlineMatch, fetchGetStats, fetchTurnLog } from "../api/fetch";

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

export const useTurnLog = (match_id: string, visible: boolean) => {
    return useQuery({
        queryKey: ['turn', match_id],
        queryFn: () => fetchTurnLog({ match_id }),
        enabled: visible && !!match_id,
        staleTime: Infinity,
    });
};

export const useOnlineMatches = () => {
    return useQuery({
        queryKey: ['onlineMatches'],
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
        queryKey: ['gametypes'],
        queryFn: fecthGetGameTypes,
    });
} 