import type React from "react";
import type { GetAgentResponse, GetGameTypeResponse } from "../api/interface";
import { Select } from "antd";

export interface AgentSelectorProps {
    agents: GetAgentResponse[],
    mode: 'multi' | 'single',
}

export const AgentSelector: React.FC<AgentSelectorProps> = ({ agents, mode, ...restProps }) => {

    const agentOptions = agents.map((agent) => {
        return {
            value: agent.name,
            label: `${agent.name}`
        };
    });

    // 3. 根据 mode 属性设置 Ant Design Select 的 mode prop
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
    gameTypes: GetGameTypeResponse[]
};

export const GameTypeSelector: React.FC<GameTypeSelectorProps> = ({ gameTypes, ...restProps }) => {
    const gameTypeOptions = gameTypes.map((gameType) => {
        return { value: gameType.game_type, label: gameType.game_type };
    })

    return (
        <Select
            options={gameTypeOptions}
            allowClear
            placeholder="Your Agent"
            {...restProps}
        />
    )
}