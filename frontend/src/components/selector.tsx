import type React from "react";
import type { GetAgentResponse, GetGameTypeResponse } from "../api/interface";
import { Select, Spin } from "antd";

export interface AgentSelectorProps {
    isAgentsLoading: boolean,
    agents?: GetAgentResponse[],
    mode: 'multi' | 'single',
    isAgentDisabled?: (agent: GetAgentResponse) => boolean;
}

export const AgentSelector: React.FC<AgentSelectorProps> = ({
    isAgentsLoading, agents, mode, isAgentDisabled, ...restProps
}) => {

    if (isAgentsLoading || !agents) {
        return (
            <Spin size="small" />
        )
    }
    const agentOptions = agents.map((agent) => {
        const isDisabled = isAgentDisabled ? isAgentDisabled(agent) : false;
        return {
            value: agent.agent_id,
            label: `${agent.name}`,
            disabled: isDisabled,
        };
    });
    const selectMode = mode === 'multi' ? 'multiple' : undefined;
    return (
        <Select
            options={agentOptions}
            allowClear
            mode={selectMode} // 应用多选或单选模式
            {...restProps}
        />
    )
}

export interface GameTypeSelectorProps {
    isGameTypeLoading: boolean,
    gameTypes?: GetGameTypeResponse[]
};

export const GameTypeSelector: React.FC<GameTypeSelectorProps> = ({
    gameTypes, isGameTypeLoading, ...restProps
}) => {

    if (isGameTypeLoading || !gameTypes) {
        return (
            <Spin size="small" />
        )
    }
    const gameTypeOptions = gameTypes.map((gameType) => {
        return { value: gameType.game_type_id, label: gameType.name };
    })
    return (
        <Select
            options={gameTypeOptions}
            allowClear
            placeholder="Game Type"
            {...restProps}
        />
    )
}

export interface PolicySelectorProps { }

export const PolicySelector: React.FC<PolicySelectorProps> = ({
    ...restProps
}) => {
    const policyOptions = [
        { value: 'Idle', label: '手动' },
        { value: 'AutoJoin', label: '自动加入' },
        { value: 'AutoNewAndJoin', label: '自动加入与自动等待' }
    ];
    return (
        <Select
            options={policyOptions}
            allowClear
            placeholder="Agent Policy"
            {...restProps}
        />
    )
}