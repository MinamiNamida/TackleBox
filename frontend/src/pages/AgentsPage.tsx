import React, { useState } from 'react';
import { Table, Button, Card, Space, message, Spin, } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
// import { useAuth } from '../context/AuthContext';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useAgents, useGameType } from '../hooks/useData';
import type { DeleteAgentPayload, GetAgentResponse } from '../api/interface';
import { fetchDeleteAgent } from '../api/fetch';
import { NewAgentModal, UpdateAgentModal } from '../components/modal';


const AgentsPage: React.FC = () => {
    // const { user } = useAuth();
    const [isNewAgentModalVisible, setIsNewAgentModalVisible] = useState(false);
    const [updateAgent, setUpdateAgent] = useState<GetAgentResponse | null>(null);
    const queryClient = useQueryClient();

    const {
        data: agents,
        isLoading: isAgentsLoading,
        isError: isAgentsError,
        error: agentsError,
    } = useAgents();

    const {
        data: gameTypes,
        isLoading: isGameTypesLoading,
        isError: isGameTypesError,
        error: gameTypeError,
    } = useGameType();

    const deleteAgentMutation = useMutation({
        mutationFn: fetchDeleteAgent,
        onSuccess: () => {
            message.success('删除成功！');
            queryClient.invalidateQueries({ queryKey: ['agents'] })
        },
        onError: (error) => {
            const errorMessage = (error as any).response?.data?.message || '更新 Agent 失败';
            message.error(errorMessage);
        }
    })

    if (isAgentsLoading || isGameTypesLoading) {
        return <Spin>加载 Agents 中...</Spin>;
    }
    if (!agents || !gameTypes || isAgentsError || isGameTypesError) {
        return <Spin>加载失败: {(agentsError || gameTypeError as Error).message || "位置"}</Spin>;
    }

    const handleDeleteAgentSubmit = (payload: DeleteAgentPayload) => {
        deleteAgentMutation.mutate(payload);
    };

    const columns: ColumnsType<GetAgentResponse> = [
        { title: 'Id', dataIndex: 'agent_id', key: 'agent_id' },
        { title: '名称', dataIndex: 'name', key: 'name', sorter: (a, b) => a.name.localeCompare(b.name) },
        { title: '类型', dataIndex: 'game_type_name', key: 'game_type_name' },
        { title: '胜场', dataIndex: 'won_games', key: 'won_games', sorter: (a, b) => (a.won_games - b.won_games) },
        { title: '场次', dataIndex: 'played_games', key: 'played_games', sorter: (a, b) => (a.played_games - b.played_games) },
        { title: '版本', dataIndex: 'version', key: 'version' },
        { title: '描述', dataIndex: 'description', key: 'description' },
        {
            title: '操作',
            key: 'action',
            render: (_text: string, record: GetAgentResponse) => (
                <Space size="small">
                    {/* 🌟 绑定 handleEdit，并将当前行数据 (record) 传入 */}
                    <Button variant="link" color='primary' onClick={() => setUpdateAgent(record)}>编辑</Button>
                    <Button variant='link' color='danger' onClick={() => handleDeleteAgentSubmit({ agent_id: record.agent_id })}>删除</Button>
                </Space>
            ),
        },
    ];

    return (
        <Card
            title="My Agents"
            extra={
                <Button type="primary" icon={<PlusOutlined />} onClick={() => setIsNewAgentModalVisible(true)}>
                    创建新 Agent
                </Button>
            }
        >
            <Table dataSource={agents || []} columns={columns} rowKey="agent_id" />

            {/* 创建 Agent 模态框 */}
            <NewAgentModal
                isGameTypeLoading={isGameTypesLoading}
                gameTypes={gameTypes}
                onCancel={() => setIsNewAgentModalVisible(false)}
                visible={isNewAgentModalVisible}
            />

            <UpdateAgentModal
                agent={updateAgent}
                isGameTypeLoading={isGameTypesLoading}
                gameTypes={gameTypes}
                onCancel={() => setUpdateAgent(null)}
                visible={updateAgent !== null}
            />

        </Card>
    );
};

export default AgentsPage;