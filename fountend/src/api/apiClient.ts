// frontend/src/api/apiClient.ts
import axios from 'axios';

const apiClient = axios.create({
    baseURL: '/api/v1',
    timeout: 10000,
    headers: {
        'Content-Type': 'application/json',
    },
});

apiClient.interceptors.request.use(config => {
    const token = localStorage.getItem('authToken');
    if (token) {
        config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
}, error => {
    return Promise.reject(error);
});


apiClient.interceptors.response.use(
    response => response,
    error => {
        const status = error.response?.status;
        const url = error.config?.url;
        if (status === 401) {

            const isAuthRoute = url && (url.includes('/auth/login') || url.includes('/auth/register'));

            if (!isAuthRoute) {
                console.error("Token expired or unauthorized access. Forcing logout.");
                localStorage.removeItem('authToken');
                // 触发页面跳转/刷新
                window.location.href = '/login';
            }
        }

        return Promise.reject(error);
    }
);

export default apiClient;