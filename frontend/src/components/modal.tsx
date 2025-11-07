import { Button, Divider, Form, Input, InputNumber, message, Modal, Radio, Result, Space, Spin } from "antd";
import type { GetAgentResponse, GetGameTypeResponse, GetMatchResponse, GetOnlineMatchResponse, JoinMatchPayload, NewAgentPayload, NewMatchPayload, TurnLogResponse, UpdateAgentPayload } from "../api/interface";
import { AgentSelector, GameTypeSelector, PolicySelector } from "./selector";
import { RocketOutlined } from "@ant-design/icons";
import { fetchJoinMatch, fetchNewAgent, fetchNewMatch, fetchTurnLog, fetchUpdateAgent } from "../api/fetch";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type React from "react";
import Paragraph from "antd/es/typography/Paragraph";
import { TurnLogSummaryTable } from "./table";

export interface NewAgentModalProps {
    isGameTypeLoading: boolean,
    gameTypes?: GetGameTypeResponse[],
    onCancel: () => void;
    visible: boolean,
}

export const NewAgentModal: React.FC<NewAgentModalProps> = ({
    isGameTypeLoading,
    gameTypes,
    onCancel,
    visible,
}) => {
    const queryClient = useQueryClient();
    const newAgentMutation = useMutation({
        mutationFn: fetchNewAgent,
        onSuccess: (_data, variants) => {
            message.success(`Agent "${variants.name} Create Successfully!`);
            queryClient.invalidateQueries({ queryKey: ['agents'] });
            onCancel();
            form.resetFields();
        },
        onError: (error) => {
            const errorMessage = (error as any).response?.data?.message || '创建 Agent 失败';
            message.error(errorMessage);
        }
    })
    const onFinish = async (payload: NewAgentPayload) => {
        newAgentMutation.mutate(payload);
    }
    const [form] = Form.useForm();
    return (
        <Modal
            open={visible}
            onCancel={onCancel}
            footer={null}
        >
            <Form
                form={form}
                layout='vertical'
                onFinish={onFinish}
            >
                <Form.Item
                    name="name"
                    label="Agent 名称"
                    rules={[{ required: true, message: '请输入名称' }]}
                >
                    <Input placeholder="My Agent" />
                </Form.Item>
                <Form.Item
                    name="game_type_id"
                    label="游戏类型"
                    rules={[{ required: true, message: '请输入类型' }]}
                >
                    <GameTypeSelector gameTypes={gameTypes} isGameTypeLoading={isGameTypeLoading} />
                </Form.Item>
                <Form.Item
                    name="version"
                    label="Agent 版本"
                    rules={[{ required: true, message: '请输入版本' }]}
                >
                    <Input placeholder="0.0.1" />
                </Form.Item>
                <Form.Item
                    name="description"
                    label="描述"
                >
                    <Input placeholder="" />
                </Form.Item>
                <Form.Item
                    name="policy"
                    label="是否自动参与游戏（暂不支持）"
                >
                    <PolicySelector />
                </Form.Item>
                <Form.Item style={{ marginTop: 24 }}>
                    <Button type="primary" htmlType="submit" icon={<RocketOutlined />}>
                        创建 Agent
                    </Button>
                </Form.Item>
            </Form>

        </Modal>
    )
}

export interface UpdateAgentModalProps {
    agent: GetAgentResponse | null,
    isGameTypeLoading: boolean,
    gameTypes?: GetGameTypeResponse[],
    onCancel: () => void;
    visible: boolean,
}


export const UpdateAgentModal: React.FC<UpdateAgentModalProps> = ({
    agent,
    isGameTypeLoading,
    gameTypes,
    onCancel,
    visible,
}) => {
    const queryClient = useQueryClient();
    const updateAgentMutation = useMutation({
        mutationFn: fetchUpdateAgent,
        onSuccess: (_data, variants) => {
            message.success(`Agent "${variants.name} Update Successfully!`);
            queryClient.invalidateQueries({ queryKey: ['agents'] });
            onCancel();
            form.resetFields();
        },
        onError: (error) => {
            const errorMessage = (error as any).response?.data?.message || '更新 Agent 失败';
            message.error(errorMessage);
        }
    })
    const [form] = Form.useForm();

    if (!agent) {
        return (
            <></>
        )
    }
    const onFinish = async (payload: UpdateAgentPayload) => {
        updateAgentMutation.mutate(payload);
    }
    const initialValues = {
        agent_id: agent.agent_id,
        name: agent.name,
        game_type_id: agent.game_type_id,
        version: agent.version,
        description: agent.description,
        policy: agent.policy
    };
    return (
        <Modal
            open={visible}
            onCancel={onCancel}
            footer={null}
        >
            <Form
                form={form}
                layout='vertical'
                onFinish={onFinish}
                initialValues={initialValues}
            >
                <Form.Item
                    name="agent_id"
                    hidden
                >
                    <Input />
                </Form.Item>
                <Form.Item
                    name="name"
                    label="Agent 名称"
                    rules={[{ required: true, message: '请输入名称' }]}
                >
                    <Input />
                </Form.Item>
                <Form.Item
                    name="game_type_id"
                    label="游戏类型"
                    rules={[{ required: true, message: '请输入类型' }]}
                >
                    <GameTypeSelector gameTypes={gameTypes} isGameTypeLoading={isGameTypeLoading} />
                </Form.Item>
                <Form.Item
                    name="version"
                    label="Agent 版本"
                    rules={[{ required: true, message: '请输入版本' }]}
                >
                    <Input />
                </Form.Item>
                <Form.Item
                    name="description"
                    label="描述"
                >
                    <Input />
                </Form.Item>
                <Form.Item
                    name="policy"
                    label="是否自动参与游戏（暂不支持）"
                >
                    <PolicySelector />
                </Form.Item>
                <Form.Item style={{ marginTop: 24 }}>
                    <Button type="primary" htmlType="submit" icon={<RocketOutlined />}>
                        更新 Agent
                    </Button>
                </Form.Item>
            </Form>

        </Modal>
    )
}


export interface NewMatchModelProps {
    isAgentsLoading: boolean,
    isGameTypeLoading: boolean,
    myAgents?: GetAgentResponse[],
    gameTypes?: GetGameTypeResponse[],
    onCancel: () => void;
    visible: boolean;
}


export const NewMatchModal: React.FC<NewMatchModelProps> = ({
    onCancel,
    visible,
    myAgents,
    isGameTypeLoading,
    isAgentsLoading,
    gameTypes
}) => {
    const queryClient = useQueryClient();
    const newMatchMutation = useMutation({
        mutationFn: fetchNewMatch,
        onSuccess: (_data, variables) => {
            message.success(`Match "${variables.name}" 创建成功!`);
            queryClient.invalidateQueries({ queryKey: ['matches'] });
            onCancel();
            form.resetFields();
        },
        onError: (error) => {
            const errorMessage = (error as any).response?.data?.message || '创建 Match 失败';
            message.error(errorMessage);
        }
    });

    const onFinish = async (payload: NewMatchPayload) => {
        newMatchMutation.mutate(payload);
    }

    const [form] = Form.useForm();
    const visibility = Form.useWatch('visibility', form);
    const initialValues = {
        total_games: 50,
    };

    return (
        <Modal
            open={visible}
            onCancel={onCancel}
            footer={null}
        >
            <Form
                form={form}
                layout="vertical"
                onFinish={onFinish}
                initialValues={initialValues}
            >
                <Form.Item
                    name="name"
                    label="比赛名称"
                    rules={[{ required: true, message: '请输入名称' }]}
                >
                    <Input placeholder="" />
                </Form.Item>

                <Form.Item name="game_type_id" label="游戏类型" rules={[{ required: true, message: '请选择游戏类型' }]}>
                    <GameTypeSelector gameTypes={gameTypes} isGameTypeLoading={isGameTypeLoading} />
                </Form.Item>

                <Form.Item name="with_agent_ids" label="选择参加的 Agent"
                    getValueFromEvent={(value) => value || []}
                >
                    <AgentSelector agents={myAgents} isAgentsLoading={isAgentsLoading} mode='multi' />
                </Form.Item>

                <Divider orientation="left">比赛配置</Divider>

                <Space size="large" style={{ display: 'flex' }}>
                    {/* <Form.Item
                        name="total_slots"
                        label="总玩家槽位"
                        tooltip="包括创建者在内，总共需要的玩家或Agent数量"
                        rules={[{ required: true, message: '请设置槽位数' }]}
                    >
                        <InputNumber min={2} max={4} style={{ width: 120 }} />
                    </Form.Item> */}

                    <Form.Item
                        name="total_games"
                        label="总局数"
                        rules={[{ required: true, message: '请设置总局数' }]}
                    >
                        <InputNumber min={1} max={1000} style={{ width: 120 }} />
                    </Form.Item>

                    {/* <Form.Item
                        name="time_limit_seconds"
                        label="每回合时间 (秒)"
                        rules={[{ required: true, message: '请设置时间限制' }]}
                    >
                        <InputNumber min={5} max={300} style={{ width: 140 }} />
                    </Form.Item> */}
                </Space>

                <Divider orientation="left">可见性与规则</Divider>

                <Form.Item name="visibility" label="房间可见性" rules={[{ required: true }]}>
                    <Radio.Group>
                        <Radio value="Public">公开 (任何人可加入)</Radio>
                        <Radio value="Private">私密 (需密码)</Radio>
                    </Radio.Group>
                </Form.Item>

                {/* 仅在选择 "Private" 时显示密码输入框 */}
                {visibility === 'Private' && (
                    <Form.Item
                        name="password"
                        label="房间密码"
                        rules={[{ required: true, message: '私密房间必须设置密码' }]}
                    >
                        <Input.Password placeholder="请输入密码" />
                    </Form.Item>
                )}

                {/* <Form.Item name="is_ranked" label="是否为排位赛" valuePropName="checked">
                    <Switch
                        checkedChildren="排位赛"
                        unCheckedChildren="休闲赛"
                    />
                </Form.Item> */}

                {/* Initial Setup 字段 (如果需要，使用 TextArea) */}
                {/* <Form.Item name="initial_setup" label="起始配置 (可选)">
                    <Input.TextArea rows={2} placeholder="输入游戏的特殊起始配置或种子" />
                </Form.Item> */}

                <Form.Item style={{ marginTop: 24 }}>
                    <Button type="primary" htmlType="submit" icon={<RocketOutlined />}>
                        创建比赛
                    </Button>
                </Form.Item>
            </Form>
        </Modal>
    );
};


export interface JoinMacthModalProps {
    onlineMatch: GetOnlineMatchResponse | null,
    isAgentsLoading: boolean,
    myAgents?: GetAgentResponse[],
    onCancel: () => void,
    visible: boolean,
}

export const JoinMacthModal: React.FC<JoinMacthModalProps> = ({
    onlineMatch,
    isAgentsLoading,
    myAgents,
    onCancel,
    visible,
}) => {
    const queryClient = useQueryClient();
    const joinMatchMutation = useMutation({
        mutationFn: fetchJoinMatch,
        onSuccess: (_data, _variables) => {
            message.success(`Match 加入成功!`);
            queryClient.invalidateQueries({ queryKey: ['onlineMatches'] });
            queryClient.invalidateQueries({ queryKey: ['matches'] });
            onCancel();
            form.resetFields();
        },
        onError: (error) => {
            const errorMessage = (error as any).response?.data?.message || '加入 Match 失败';
            message.error(errorMessage);
        }
    })
    const [form] = Form.useForm();
    const onFinish = (payload: JoinMatchPayload) => {
        joinMatchMutation.mutate(payload)
    }

    if (!onlineMatch) {
        return (<></>)
    }

    return (
        <Modal
            open={visible}
            onCancel={onCancel}
            footer={null}
        >
            <Form
                form={form}
                layout="vertical"
                onFinish={onFinish}
            >
                <Form.Item
                    name="match_id"
                    hidden
                    initialValue={onlineMatch.match_id} // 通过 initialValue 确保值被设置
                >
                    <Input />
                </Form.Item>

                <Form.Item label="比赛名称"> {/* 仅用于展示，移除 name 避免警告 */}
                    <span style={{ fontWeight: 'bold' }}>
                        {onlineMatch.match_name}
                    </span>
                </Form.Item>

                <Form.Item name="agent_ids">
                    <AgentSelector
                        agents={myAgents}
                        isAgentsLoading={isAgentsLoading}
                        mode='multi'
                        isAgentDisabled={(agent) => {
                            return agent.status !== 'Ready' || agent.game_type_id !== onlineMatch.game_type_id
                        }}
                    />
                </Form.Item>

                <Form.Item name="password" hidden={!onlineMatch.with_password}>
                    <Input.Password />
                </Form.Item>

                <Form.Item style={{ marginTop: 24 }}>
                    <Button
                        type="primary"
                        htmlType="submit"
                        icon={<RocketOutlined />}
                        loading={joinMatchMutation.isPending}
                        disabled={joinMatchMutation.isPending}
                    >
                        {joinMatchMutation.isPending ? '加入中...' : '加入比赛'}
                    </Button>
                </Form.Item>
            </Form>
        </Modal>
    );
}

export interface GetTurnLogModalProps {
    match: GetMatchResponse | null,
    onCancel: () => void,
    visible: boolean,
}
export const GetTurnLogModal: React.FC<GetTurnLogModalProps> = ({ match, onCancel, visible }) => {
    // 使用可选链安全地获取 match_id 和 match_name
    const match_id = match?.match_id;
    const match_name = match?.match_name || match_id; // 优先使用名称，否则使用ID

    const isEnabled = visible && !!match_id;

    // 假设 useQuery 成功时返回 TurnLogResponse[]
    const { data: turnLogs, isLoading, isError, error } = useQuery<TurnLogResponse[], Error>({
        queryKey: ['turn', match_id],
        queryFn: () => {
            if (!match_id) {
                // 运行时检查，但 enabled 保证了它通常不会被执行
                throw new Error("Query enabled but match_id is missing.");
            }
            return fetchTurnLog({ match_id });
        },
        enabled: isEnabled,
        staleTime: Infinity,
    });

    // 如果 match_id 丢失，直接返回空，通常是父组件传递错误
    if (!match_id) {
        return <></>;
    };

    const renderContent = () => {
        if (isLoading) {
            return <div style={{ textAlign: 'center', padding: '50px' }}><Spin size="large" /></div>;
        }

        if (isError) {
            const errorMessage = (error as Error).message;
            return (
                <Result
                    status="error"
                    title="加载比赛日志失败"
                    subTitle={<Paragraph code>{errorMessage}</Paragraph>}
                />
            );
        }

        if (turnLogs && turnLogs.length > 0) {
            // 🚀 集成 TurnLogSummaryTable
            return (
                <TurnLogSummaryTable
                    logs={turnLogs}
                    matchName={match_name}
                />
            );
        }

        // 没有日志数据或数据为空
        return (
            <Result
                title="暂无日志记录"
                subTitle="当前比赛可能尚未开始或尚未产生回合日志。"
            />
        );
    };

    return (
        <Modal
            title={`比赛日志: ${match_name}`}
            open={visible}
            onCancel={onCancel}
            footer={null}
            width={1000} // 增大宽度以容纳表格
        >
            {renderContent()}
        </Modal>
    );
}