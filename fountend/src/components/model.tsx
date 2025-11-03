import { Button, Divider, Form, Input, InputNumber, message, Modal, Radio, Space, Spin, Switch } from "antd";
import type { GetAgentResponse, GetGameTypeResponse, GetOnlineMatchResponse, JoinMatchPayload, NewMatchPayload } from "../api/interface";
import { AgentSelector, GameTypeSelector } from "./selector";
import { RocketOutlined } from "@ant-design/icons";
import { fetchJoinMatch, fetchNewMatch } from "../api/fetch";
import { useMutation, useQueryClient } from "@tanstack/react-query";




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
        onSuccess: (data, variables) => {
            message.success(`Match "${variables.name}" 创建成功!`);
            queryClient.invalidateQueries({ queryKey: ['matches'] });
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

                <Form.Item name="game_type" label="游戏类型" rules={[{ required: true, message: '请选择游戏类型' }]}>
                    {isGameTypeLoading || gameTypes === undefined ? (
                        <Spin size="small" />
                    ) : (
                        <GameTypeSelector gameTypes={gameTypes} />
                    )}
                </Form.Item>

                <Form.Item name="with_agent_names" label="选择参加的 Agent">
                    {isAgentsLoading || myAgents === undefined ? (
                        <Spin size="small" />
                    ) : (
                        <AgentSelector agents={myAgents} mode='multi' />
                    )}
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
        onSuccess: (data, variables) => {
            message.success(`Match "${variables.match_name}" 加入成功!`);
            queryClient.invalidateQueries({ queryKey: ['matches'] });
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
        return
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
                    name="match_name"
                    hidden
                    initialValue={onlineMatch.match_name} // 通过 initialValue 确保值被设置
                >
                    <Input />
                </Form.Item>

                <Form.Item label="比赛名称"> {/* 仅用于展示，移除 name 避免警告 */}
                    <span style={{ fontWeight: 'bold' }}>
                        {onlineMatch.match_name}
                    </span>
                </Form.Item>

                <Form.Item name="agent_name">
                    {isAgentsLoading || myAgents === undefined ? (
                        <Spin size="small" />
                    ) : (
                        <AgentSelector agents={myAgents} mode='single' />
                    )}
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