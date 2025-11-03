import React, { useState } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';

// 页面组件
import MainLayout from './pages/MainLayout';
import AuthPage from './pages/AuthPage';
import AgentsPage from './pages/AgentsPage';
import MatchesPage from './pages/MatchesPage';
import { AuthProvider, useAuth } from './context/AuthContext';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

const AppContent: React.FC = () => {
  const { isLoggedIn } = useAuth();

  return (
    <Routes>

      {/* 认证路由 (未登录可见) */}
      <Route path="/login" element={<AuthPage />} />
      <Route path="/register" element={<AuthPage />} />

      {/* 主应用布局路由 (需要登录) */}
      <Route element={<MainLayout />}>
        {/* 检查是否已登录，如果未登录，重定向到 /login */}
        <Route path="/" element={isLoggedIn ? <Navigate to="/agents" replace /> : <Navigate to="/login" replace />} />

        {/* 需要登录才能访问的页面 */}
        {isLoggedIn ? (
          <>
            <Route path="/agents" element={<AgentsPage />} />
            <Route path="/matches" element={<MatchesPage />} />
            <Route path="/profile" element={<div>个人信息页面...</div>} />
          </>
        ) : (
          <Route path="*" element={<Navigate to="/login" replace />} />
        )}

      </Route>

    </Routes>
  );
};

const queryClient = new QueryClient();

const App: React.FC = () => {
  return (
    <React.StrictMode>
      <BrowserRouter>
        <AuthProvider>
          <QueryClientProvider client={queryClient}>
            <AppContent />
          </QueryClientProvider>
        </AuthProvider>
      </BrowserRouter>
    </React.StrictMode>
  );
};

export default App;