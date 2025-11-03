import React, { useState } from 'react';
import { Table, Button, Card, Tag, Modal, Form, Input, Select, Space, message, Spin, notification } from 'antd';
import { CopyOutlined, PlusOutlined, UploadOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { useAuth } from '../context/AuthContext';
import { useMutation, useQueryClient, useQuery } from '@tanstack/react-query';
import { useAgents, useGameType } from '../hooks/useData';
import type { DeleteAgentPayload, GetAgentResponse, NewAgentPayload, UpdateAgentPayload } from '../api/interface';
import { fetchDeleteAgent, fetchNewAgent, fetchUpdateAgent } from '../api/fetch';


const { Option } = Select;

const AgentsPage: React.FC = () => {
    const { user } = useAuth();
    const [isCreateAgentModalVisible, setIsCreateAgentModalVisible] = useState(false);
    const [isUpdateAgentModalVisible, setIsUpdateAgentModalVisible] = useState(false);
    const [form] = Form.useForm();
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

    const createAgentMutation = useMutation({
        mutationFn: fetchNewAgent,
        onSuccess: (newAgentId, variables) => {
            message.success(`Agent "${variables.name}" 创建成功! ID: ${newAgentId}`);
            queryClient.invalidateQueries({ queryKey: ['agents'] });
            setIsCreateAgentModalVisible(false);
        },
        onError: (error) => {
            const errorMessage = (error as any).response?.data?.message || '创建 Agent 失败';
            message.error(errorMessage);
        }
    });

    const updateAgentMutation = useMutation({
        mutationFn: fetchUpdateAgent,
        onSuccess: () => {
            message.success(`更新成功!`);
            queryClient.invalidateQueries({ queryKey: ['agents'] });
            setIsUpdateAgentModalVisible(false);
        },
        onError: (error) => {
            const errorMessage = (error as any).response?.data?.message || '更新 Agent 失败';
            message.error(errorMessage);
        }
    });

    const deleteAgentMutation = useMutation({
        mutationFn: fetchDeleteAgent,
        onSuccess: () => {
            message.success('删除成功！');
            queryClient.invalidateQueries({ queryKey: ['agents'] })
            setIsUpdateAgentModalVisible(false);
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

    const handleNewAgentSubmit = (payload: NewAgentPayload) => {
        createAgentMutation.mutate(payload);

    };

    const handleDeleteAgentSubmit = (payload: DeleteAgentPayload) => {
        deleteAgentMutation.mutate(payload);
    };

    const handleUpdateAgentSubmit = (payload: UpdateAgentPayload) => {
        updateAgentMutation.mutate(payload);
    }

    const handleEdit = (name: string) => {
        const agent = agents?.find((a) => a.name == name);
        if (agent === undefined) {
            message.error('选中了一个不存在的agent?', 0);

            return;
        }
        form.setFieldsValue({
            name: agent.name,
            game_type: agent.game_type,
            version: agent.version,
            description: agent.description,
        });
        setIsUpdateAgentModalVisible(true);
    };

    const handleDelete = (name: string) => {
        handleDeleteAgentSubmit({ name })
    }

    const handleCancel = () => {
        setIsUpdateAgentModalVisible(false);
        form.resetFields(); // 重置表单所有字段
    };

    const columns: ColumnsType<GetAgentResponse> = [
        { title: '名称', dataIndex: 'name', key: 'name', sorter: (a, b) => a.name.localeCompare(b.name) },
        { title: '类型', dataIndex: 'game_type', key: 'game_type' },
        { title: '胜场', dataIndex: 'won_games', key: 'won_games', sorter: (a, b) => (a.won_games - b.won_games) },
        { title: '场次', dataIndex: 'played_games', key: 'played_games', sorter: (a, b) => (a.played_games - b.played_games) },
        { title: '版本', dataIndex: 'version', key: 'version' },
        { title: '描述', dataIndex: 'description', key: 'description' },
        {
            title: '操作',
            key: 'action',
            render: (text: string, record: GetAgentResponse) => (
                <Space size="small">
                    {/* 🌟 绑定 handleEdit，并将当前行数据 (record) 传入 */}
                    <Button variant="link" color='primary' onClick={() => handleEdit(record.name)}>编辑</Button>
                    <Button variant='link' color='danger' onClick={() => handleDelete(record.name)}>删除</Button>
                </Space>
            ),
        },
    ];

    return (
        <Card
            title="My Agents"
            extra={
                <Button type="primary" icon={<PlusOutlined />} onClick={() => setIsCreateAgentModalVisible(true)}>
                    创建新 Agent
                </Button>
            }
        >
            <Table dataSource={agents || []} columns={columns} rowKey="id" />

            {/* 创建 Agent 模态框 */}
            <Modal
                title="创建新Agent"
                open={isCreateAgentModalVisible}
                onCancel={() => setIsCreateAgentModalVisible(false)}
                footer={null}
            >
                <Form form={form} layout="vertical" onFinish={handleNewAgentSubmit}>
                    <Form.Item name="name" label="Agent名称" rules={[{ required: true, message: '请输入名称' }]}>
                        <Input placeholder="例如: MyAgent" />
                    </Form.Item>
                    <Form.Item name="game_type" label="游戏类型" rules={[{ required: true, message: '请选择游戏类型' }]}>
                        <Select placeholder="选择游戏类型">
                            {
                                gameTypes.map((item) => (
                                    <Option value={item.game_type}>
                                        {item.game_type}
                                    </Option>
                                ))
                            }
                        </Select>
                    </Form.Item>
                    <Form.Item name="version" label="版本号" initialValue="1.0.0" rules={[{ required: true, message: '请输入版本号' }]}>
                        <Input placeholder="例如: 1.0.0" />
                    </Form.Item>
                    <Form.Item name="description" label="描述">
                        <Input.TextArea rows={2} />
                    </Form.Item>
                    <Form.Item>
                        <Button type="primary" htmlType="submit" block icon={<UploadOutlined />}>
                            创建 Agent
                        </Button>
                    </Form.Item>
                </Form>
            </Modal>

            <Modal
                title="更新Agent"
                open={isUpdateAgentModalVisible}
                onCancel={handleCancel}
                footer={null}
            >
                <Form form={form} layout="vertical" onFinish={handleUpdateAgentSubmit}>
                    <Form.Item name="name" label="Agent 名称" rules={[{ required: true, message: '请输入名称' }]}>
                        <Input placeholder="" />
                    </Form.Item>
                    <Form.Item name="game_type" label="游戏类型" rules={[{ required: true, message: '请选择游戏类型' }]}>
                        <Select placeholder="">
                            {
                                gameTypes.map((item) => (
                                    <Option value={item.game_type}>
                                        {item.game_type}
                                    </Option>
                                ))
                            }
                        </Select>
                    </Form.Item>
                    <Form.Item name="version" label="版本号" initialValue="1.0.0" rules={[{ required: true, message: '请输入版本号' }]}>
                        <Input placeholder="例如: 1.0.0" />
                    </Form.Item>
                    <Form.Item name="description" label="描述">
                        <Input.TextArea rows={2} />
                    </Form.Item>
                    <Form.Item>
                        <Button type="primary" htmlType="submit" block icon={<UploadOutlined />}>
                            修改 Agent
                        </Button>
                    </Form.Item>
                </Form>
            </Modal>

        </Card>
    );
};

export default AgentsPage;