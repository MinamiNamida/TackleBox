import React, { useState } from 'react';
import { Layout, Menu, theme, Button, Space, Typography } from 'antd';
import { UserOutlined, DeploymentUnitOutlined, TrophyOutlined, LogoutOutlined } from '@ant-design/icons';
import { Outlet, useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';

const { Header, Content, Sider } = Layout;
const { Title } = Typography;

// 侧边栏菜单项
const userMenuItems = [
    { key: '/agents', icon: <DeploymentUnitOutlined />, label: '我的 Agents' },
    { key: '/matches', icon: <TrophyOutlined />, label: '我的 Match' },
    { key: '/profile', icon: <UserOutlined />, label: '个人信息' },
];


const MainLayout: React.FC = () => {
    const { isLoggedIn, user, logout } = useAuth();
    const navigate = useNavigate();
    // 模拟当前选中的菜单项，方便高亮
    const [current, setCurrent] = useState(window.location.pathname);

    const handleLogout = () => {
        logout(); // 🌟 调用 Context 提供的 logout 方法
        navigate('/login');
        // window.location.reload() 不再需要，Context 会自动更新状态
    };

    const handleMenuClick = (e: { key: string }) => {
        setCurrent(e.key);
        navigate(e.key);
    };

    // -------------------- 布局返回 --------------------

    // 未登录时只显示简单的内容
    if (!isLoggedIn) {
        return (
            <Layout style={{ minHeight: '100vh', padding: 20 }}>
                <Content><Outlet /></Content>
            </Layout>
        );
    }

    // 登录后的主要管理界面布局
    return (
        <Layout style={{ minHeight: '100vh' }}>
            <Sider width={200} theme="dark">
                <Title level={4} style={{ color: 'white', textAlign: 'center', margin: '16px 0' }}>
                    Tackle Box
                </Title>
                <Menu
                    theme="dark"
                    mode="inline"
                    selectedKeys={[current]}
                    onClick={handleMenuClick}
                    items={userMenuItems}
                />
            </Sider>

            <Layout className="site-layout">
                <Header style={{ background: theme.useToken().token.colorBgContainer, padding: '0 24px', display: 'flex', justifyContent: 'flex-end', alignItems: 'center' }}>
                    <Space>
                        <span style={{ marginRight: 16 }}>欢迎，{user?.username}</span>
                        <Button type="text" icon={<LogoutOutlined />} onClick={handleLogout}>
                            退出登录
                        </Button>
                    </Space>
                </Header>

                <Content style={{ margin: '16px', padding: '16px', background: theme.useToken().token.colorBgContainer, borderRadius: '8px' }}>
                    {/* Outlet 用于渲染子路由内容，例如 /agents 页面 */}
                    <Outlet />
                </Content>
            </Layout>
        </Layout>
    );
};

export default MainLayout;