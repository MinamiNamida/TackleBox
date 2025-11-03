import React, { useState, useMemo } from 'react';
import {
    Card,
    List,
    Tag,
    Space,
    Avatar,
    Button,
    message,
    Divider,
    Form,
    Modal,
    Spin,
    Result,
    Tabs,
    Input,
    Select
} from 'antd';
import {
    FieldTimeOutlined,
    GlobalOutlined,
    PlusOutlined,
    RocketOutlined,
    TeamOutlined,
    UserOutlined
} from '@ant-design/icons';
import type { SelectProps, TabsProps } from 'antd';
import { useAgents, useGameType, useMatches, useOnlineMatches } from '../hooks/useData';
import type { GetAgentResponse, GetGameTypeResponse, GetMatchResponse, GetOnlineMatchResponse, NewMatchPayload } from '../api/interface';
import Title from 'antd/es/typography/Title';
import { AgentSelector, GameTypeSelector } from '../components/selector';
import { JoinMacthModal, NewMatchModal } from '../components/model';

// 排序选项
const sortOptions: SelectProps['options'] = [
    { label: '按时间最新', value: 'time_desc' },
    { label: '按时间最旧', value: 'time_asc' },
];

// --- 筛选和排序的列表组件 ---
const MatchesPage: React.FC = () => {

    // --- 1. 状态管理：保存筛选和排序条件 ---
    const [gameFilter, setGameFilter] = useState('all');
    const [statusFilter, setStatusFilter] = useState('Waiting'); // 默认只看等待中的
    const [sortOrder, setSortOrder] = useState('time_desc');
    const [searchText, setSearchText] = useState('');

    const [matchCreater, setMatchCreater] = useState(false);
    const [matchJoiner, setMatchJoiner] = useState<null | GetOnlineMatchResponse>(null);

    const {
        data: myMatches,
        isLoading: myMatchesLoading,
        isError: myMatchesError
    } = useMatches();

    const {
        data: onlineMatches,
        isLoading: onlineMatchesLoading,
        isError: onlineMatchesError,
        refetch: refetchOnlineMatches // (可选) 允许刷新网络比赛
    } = useOnlineMatches();


    const {
        data: agents,
        isLoading: isAgentLoading,
        isError: isAgentError,
    } = useAgents();

    const {
        data: gameTypes,
        isLoading: isGameTypesLoading,
        isError: isGameTypesError,
    } = useGameType();

    const myMatchesTabContent = (
        <>
            <Title level={4}>我的比赛历史</Title>

            {myMatchesLoading && <Spin tip="加载中..." />}

            {myMatchesError && <Result status="error" title="加载我的比赛失败" />}

            {!myMatchesLoading && !myMatchesError && (
                <List
                    itemLayout="horizontal"
                    dataSource={myMatches || []}
                    renderItem={(item: GetMatchResponse) => {

                        let statusColor: string;
                        switch (item.status) {
                            case 'Completed': statusColor = 'blue'; break;
                            case 'Running': statusColor = 'processing'; break;
                            case 'Pending': statusColor = 'warning'; break;
                            default: statusColor = 'default';
                        }

                        return (
                            <List.Item
                                key={item.id}
                                actions={[<Button key="details" type="link">查看日志</Button>]}
                            >
                                <List.Item.Meta
                                    avatar={<Avatar icon={<TeamOutlined />} />}
                                    title={<Space>{item.name} <Tag color={statusColor}>{item.status}</Tag></Space>}
                                    description={
                                        <Space size="large">
                                            <span>游戏: {item.game_type}</span>
                                            {/* 仅在 "Completed" 状态下显示胜利者 */}
                                            {item.status === 'Completed' && (
                                                <span>结果: {item.winner_agent_name} 胜</span>
                                            )}
                                            <span style={{ color: '#999' }}>
                                                <FieldTimeOutlined /> {new Date(item.start_time).toLocaleString()}
                                            </span>
                                        </Space>
                                    }
                                />
                            </List.Item>
                        );
                    }}
                />
            )}
        </>
    );

    const renderPublicMatchList = () => {
        if (onlineMatchesLoading) {
            return <Spin tip="正在刷新比赛大厅..." />;
        }

        if (onlineMatchesError) {
            return <Result status="error" title="加载网络比赛失败" subTitle="请稍后重试" />;
        }

        const waitingMatches = (onlineMatches || []).filter(
            (match: GetOnlineMatchResponse) => match.status === 'Pending'
        );

        if (waitingMatches.length === 0) {
            return <p>当前没有等待加入的比赛。</p>;
        }

        return (

            <List
                itemLayout="horizontal"
                dataSource={waitingMatches}
                renderItem={(item: GetOnlineMatchResponse) => (
                    <List.Item
                        key={item.match_id}
                        actions={[
                            <Button
                                key="action"
                                type="primary"
                                disabled={item.status !== 'Pending'}
                                onClick={() => setMatchJoiner(item)}
                            >
                                加入比赛
                            </Button>
                        ]}
                    >
                        <List.Item.Meta
                            avatar={<Avatar style={{ backgroundColor: '#1890ff' }} icon={<GlobalOutlined />} />}
                            title={<Space>{item.match_name} ({item.creater_name})</Space>}
                            description={
                                <Space>
                                    <span>房主: {item.creater_name}</span>
                                    <Tag color="green">{item.status}</Tag>
                                    {/* 注意：GetMatchResponse 中没有 'players' 字段，所以我们隐藏它 */}
                                    {/* <span><UserOutlined /> {item.players}</span> */}
                                </Space>
                            }
                        />
                    </List.Item>
                )}
            />
        );
    }

    const items: TabsProps['items'] = [
        {
            key: '1',
            label: '我 创建/参加的比赛',
            children: myMatchesTabContent
        },
        {
            key: '2',
            label: '在线比赛',
            children: renderPublicMatchList()
        },
    ];

    return (
        <Card
            title="比赛中心"
            extra={
                <Button
                    type="primary"
                    icon={<PlusOutlined />}
                    onClick={() => setMatchCreater(true)}
                >
                    创建比赛
                </Button>
            }
        >
            <Tabs
                defaultActiveKey="1"
                items={items}
            />

            {/* ✅ 渲染 Modal */}
            <NewMatchModal
                isAgentsLoading={isAgentLoading}
                isGameTypeLoading={isGameTypesLoading}
                myAgents={agents}
                gameTypes={gameTypes}
                visible={matchCreater}
                onCancel={() => setMatchCreater(false)}
            />
            <JoinMacthModal
                onlineMatch={matchJoiner}
                isAgentsLoading={isAgentLoading}
                myAgents={agents}
                onCancel={() => setMatchJoiner(null)}
                visible={matchJoiner !== null}
            />
        </Card>
    );

};


export default MatchesPage;