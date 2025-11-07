import React, { createContext, useContext, useState, useEffect } from 'react';
import type { ReactNode } from 'react';
import { fetchGetUser, fetchLogin } from '../api/fetch'; // 引入你的 API


interface UserProfile {
    username: string;
    created_at: string;
}

// 1. 定义 Context 的值类型
interface AuthContextType {
    isLoggedIn: boolean;
    user: UserProfile | null
    // login 函数只需要接收用户名和密码，内部调用 API
    login: (username: string, password: string) => Promise<void>;
    logout: () => void;
}

// 2. 创建 Context
const AuthContext = createContext<AuthContextType | undefined>(undefined);

// 3. 检查 localStorage 的初始状态
const getInitialAuthStatus = () => {
    return !!localStorage.getItem('authToken');
};

// 4. Provider 组件
export const AuthProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const [isLoggedIn, setIsLoggedIn] = useState(getInitialAuthStatus());
    const [user, setUser] = useState<UserProfile | null>(null);

    const login = async (username: string, password: string) => {
        const resp = await fetchLogin({ username, password });
        localStorage.setItem('authToken', resp.token);
        setIsLoggedIn(true);
    };

    const logout = () => {
        localStorage.removeItem('authToken');
        setIsLoggedIn(false);
    };

    useEffect(() => {
        if (isLoggedIn && !user) {
            fetchGetUser().then(setUser).catch(logout); // 假设 fetchUserProfile 是你的 API
        }
    }, [isLoggedIn, user, logout]);

    return (
        <AuthContext.Provider value={{ isLoggedIn, user, login, logout }}>
            {children}
        </AuthContext.Provider>
    );
};

// 5. 自定义 Hook (useAuth)
export const useAuth = () => {
    const context = useContext(AuthContext);
    if (context === undefined) {
        throw new Error('useAuth must be used within an AuthProvider');
    }
    return context;
};