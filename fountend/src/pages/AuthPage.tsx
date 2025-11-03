import React, { useState } from 'react';
import { Card, Tabs, Form, Input, Button, message, Space, Typography } from 'antd';
import { UserOutlined, LockOutlined, MailOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { fetchLogin, fetchRegister } from '../api/fetch';
import { useAuth } from '../context/AuthContext';
import type { LoginPayload, RegisterPayload } from '../api/interface';

const { Title } = Typography;

const AuthPage: React.FC = () => {
    const navigate = useNavigate();
    const { login: globalLogin } = useAuth();
    const [activeKey, setActiveKey] = useState('1');

    const onLoginFinish = async (payload: LoginPayload) => {
        const hide = message.loading('登录中...', 0);

        try {
            await globalLogin(payload.username, payload.password);

            message.success('登录成功!');
            navigate('/agents');
        } catch (e: any) {
            const errorMessage = e.response?.data?.message || '操作失败，请检查网络或账号信息';
            message.error(errorMessage);
        } finally {
            hide();
        }
    };

    const onRegisterFinish = async (payload: RegisterPayload) => {
        const hide = message.loading('注册中...', 0);
        try {
            await fetchRegister(payload)
            message.success('注册成功，转入登录页面。');
            setActiveKey('1')
        } catch (e: any) {
            const errorMessage = e.response?.data?.message || '操作失败，请检查网络状态';
            message.error(errorMessage);
        } finally {
            hide();
        }
    }

    const loginForm = (
        <Form onFinish={(payload: LoginPayload) => onLoginFinish(payload)}>
            <Form.Item name="username" rules={[{ required: true, message: '请输入用户名!' }]}>
                <Input prefix={<UserOutlined />} placeholder="用户名" />
            </Form.Item>
            <Form.Item name="password" rules={[{ required: true, message: '请输入密码!' }]}>
                <Input.Password prefix={<LockOutlined />} placeholder="密码" />
            </Form.Item>
            <Form.Item>
                <Button type="primary" htmlType="submit" style={{ width: '100%' }}>
                    登录
                </Button>
            </Form.Item>
        </Form>
    );

    const registerForm = (
        <Form onFinish={(payload: RegisterPayload) => onRegisterFinish(payload)}>
            <Form.Item name="username" rules={[{ required: true, message: '请输入用户名!' }]}>
                <Input prefix={<UserOutlined />} placeholder="用户名" />
            </Form.Item>
            <Form.Item name="email" rules={[{ type: 'email', required: true, message: '请输入正确的邮箱!' }]}>
                <Input prefix={<MailOutlined />} placeholder="邮箱" />
            </Form.Item>
            <Form.Item name="password" rules={[{ required: true, message: '设置密码!' }]}>
                <Input.Password prefix={<LockOutlined />} placeholder="密码" />
            </Form.Item>
            <Form.Item name="confirm_password" dependencies={['password']} hasFeedback rules={[
                { required: true, message: '请确认密码!' },
                ({ getFieldValue }) => ({
                    validator(_, value) {
                        if (!value || getFieldValue('password') === value) {
                            return Promise.resolve();
                        }
                        return Promise.reject(new Error('两次输入的密码不一致!'));
                    },
                }),
            ]}>
                <Input.Password prefix={<LockOutlined />} placeholder="确认密码" />
            </Form.Item>
            <Form.Item>
                <Button type="primary" htmlType="submit" style={{ width: '100%' }}>
                    注册
                </Button>
            </Form.Item>
        </Form>
    );

    const items = [
        { label: '登录', key: '1', children: loginForm },
        { label: '注册', key: '2', children: registerForm },
    ];

    return (
        <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: '100vh', background: '#f0f2f5' }}>
            <Card style={{ width: 400 }}>
                <Title level={3} style={{ textAlign: 'center', marginBottom: 24 }}>Agent Platform</Title>
                <Tabs
                    defaultActiveKey="1"
                    activeKey={activeKey}
                    onChange={setActiveKey}
                    items={items}
                />
            </Card>
        </div>
    );
};

export default AuthPage;