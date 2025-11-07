import React from 'react';
import { Table, Tag, Button, Card, Typography, Tooltip } from 'antd';
import { DownloadOutlined, NumberOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';

const { Title, Text } = Typography;

export interface TurnLogResponse {
    turn_id: string;
    match_id: string;
    log: string;
    i_turn: number;
    score_deltas: Record<string, number>;
}

interface TurnLogSummaryProps {
    logs: TurnLogResponse[];
    matchName?: string;
}


const handleDownload = (logContent: string, filename: string) => {
    const blob = new Blob([logContent], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.setAttribute('download', filename);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
};

export const TurnLogSummaryTable: React.FC<TurnLogSummaryProps> = ({ logs, matchName = '当前比赛' }) => {
    const handleDownloadAllLogs = () => {
        if (logs.length === 0) return;

        const dataToDownload = logs.map(log => {
            let detailedLog;

            if (typeof log.log === 'object' && log.log !== null) {
                detailedLog = log.log;
            } else {
                detailedLog = log.log;
            }

            return {
                turn_id: log.turn_id,
                i_turn: log.i_turn,
                score_deltas: log.score_deltas,
                detailed_log_content: detailedLog,
            };
        });

        const allLogsContent = JSON.stringify(dataToDownload, null, 2);

        const matchId = logs[0].match_id;
        const filename = `${matchId}_full_match_log.json`;

        handleDownload(allLogsContent, filename);
    };

    const handleDownloadSingle = (logContent: any, turnIndex: number, matchId: string) => {
        let contentToDownload: string;

        if (typeof logContent === 'object' && logContent !== null) {
            contentToDownload = JSON.stringify(logContent, null, 2);
        } else {
            contentToDownload = String(logContent);
        }

        const filename = `${matchId}_turn_${turnIndex}_log.json`;
        handleDownload(contentToDownload, filename);
    };
    const columns: ColumnsType<TurnLogResponse> = [
        {
            title: <NumberOutlined />,
            dataIndex: 'i_turn',
            key: 'i_turn',
            width: 80,
            sorter: (a, b) => a.i_turn - b.i_turn,
            render: (text) => <Text strong>第 {text + 1} 回合</Text>,
        },

        {
            title: '得分变化 (Agent: Δ分)',
            key: 'score_deltas',
            render: (_, record) => (
                <>
                    {Object.entries(record.score_deltas).map(([agentId, delta]) => {
                        const isPositive = delta > 0;
                        const isNegative = delta < 0;

                        let color: string = 'default';
                        if (isPositive) color = 'success';
                        if (isNegative) color = 'error';

                        return (
                            <Tooltip title={`Agent ID: ${agentId}`} key={agentId}>
                                <Tag color={color} style={{ margin: '4px 4px 4px 0' }}>
                                    {agentId.substring(0, 8)}: {delta > 0 ? `+${delta}` : delta}
                                </Tag>
                            </Tooltip>
                        );
                    })}
                </>
            ),
        },

        {
            title: '操作',
            key: 'action',
            width: 150,
            render: (_, record) => (
                <Button
                    icon={<DownloadOutlined />}
                    onClick={() => handleDownloadSingle(record.log, record.i_turn, record.match_id)}
                    size="small"
                >
                    下载详细日志
                </Button>
            ),
        },
    ];

    return (
        <Card
            title={<Title level={4}>📚 {matchName} 比赛回合摘要</Title>}
            style={{ marginBottom: 20 }}
            extra={
                <Button
                    type="primary"
                    icon={<DownloadOutlined />}
                    onClick={handleDownloadAllLogs}
                    disabled={logs.length === 0}
                >
                    下载全部日志 ({logs.length} 回合)
                </Button>
            }
        >
            <Table
                dataSource={logs}
                columns={columns}
                rowKey="turn_id" // 使用唯一的 turn_id 作为 key
                pagination={logs.length > 10 ? { pageSize: 10 } : false} // 超过10条时显示分页
                size="middle"
            />
        </Card>
    );
};